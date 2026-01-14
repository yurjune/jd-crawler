use crate::crawler::{JobCrawler, JobDetailCrawler, JobListCrawler};
use crate::user_agent::get_random_user_agent;
use crate::{Job, Result};
use headless_chrome::Tab;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use scraper::{Html, Selector};
use std::sync::Arc;
use std::time::Duration;

const JOB_CATEGORY_DEVELOPMENT: u32 = 518;
const JOB_SUBCATEGORY_FRONTEND: u32 = 669;

pub struct WantedClient {
    base_url: String,
    num_threads: usize,
}

impl WantedClient {
    pub fn new(num_threads: usize) -> Self {
        Self {
            base_url: "https://www.wanted.co.kr".to_string(),
            num_threads,
        }
    }

    fn build_url(&self, min_years: u8, max_years: u8) -> String {
        format!(
            "{}/wdlist/{}/{}?country=kr&job_sort=job.recommend_order&years={}&years={}&locations=all",
            self.base_url, JOB_CATEGORY_DEVELOPMENT, JOB_SUBCATEGORY_FRONTEND, min_years, max_years
        )
    }

    pub fn fetch_frontend_jobs(
        &self,
        min_years: u8,
        max_years: u8,
        max_page: usize,
    ) -> Result<Vec<Job>> {
        let url = self.build_url(min_years, max_years);
        let max_scroll = if max_page > 0 { max_page - 1 } else { 0 };
        let jobs = self.fetch_all_jobs(&url, max_scroll)?;

        println!("\n상세 정보 수집 시작...");

        let browsers: Vec<_> = (0..self.num_threads)
            .map(|_| {
                let thread_id = std::thread::current().id();
                let user_agent = get_random_user_agent(thread_id);
                self.create_browser(user_agent)
            })
            .collect::<Result<Vec<_>>>()?;

        let pool = ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .build()?;

        let jobs_with_details: Vec<Job> = pool.install(|| {
            jobs.par_iter()
                .enumerate()
                .map(|(idx, job)| {
                    let thread_id = std::thread::current().id();
                    let browser = &browsers[idx % self.num_threads];
                    match self.fetch_job_detail_with_browser(browser, &job.url) {
                        Ok((deadline, location)) => {
                            println!("[{:?}] [{}] 완료: {}", thread_id, idx, job.title);
                            job.clone().with_details(deadline, location)
                        }
                        Err(e) => {
                            eprintln!("[{:?}] [{}] 실패 ({}): {}", thread_id, idx, job.title, e);
                            job.clone()
                        }
                    }
                })
                .collect()
        });

        Ok(jobs_with_details)
    }

    fn fetch_job_detail_with_browser(
        &self,
        browser: &headless_chrome::Browser,
        url: &str,
    ) -> Result<(Option<String>, Option<String>)> {
        let tab = browser.new_tab()?;

        tab.navigate_to(url)?;
        tab.wait_for_element("body")?;
        std::thread::sleep(Duration::from_secs(2));

        let html = tab.get_content()?;

        let deadline = self.extract_deadline(&html);
        let location = self.extract_location(&html);

        std::thread::sleep(Duration::from_secs(1));

        Ok((deadline, location))
    }
}

impl JobCrawler for WantedClient {
    fn wait_for_page_load(&self, tab: &Arc<Tab>) -> Result<()> {
        println!("페이지 로딩 대기 중...");
        tab.wait_for_element("body")?;
        tab.wait_for_element(r#"[data-cy="job-list"]"#)?;
        std::thread::sleep(Duration::from_secs(3));
        Ok(())
    }
}

impl JobListCrawler for WantedClient {
    fn parse_all_jobs(&self, html: &str) -> Result<Vec<Job>> {
        let document = Html::parse_document(html);

        let body_selector = Selector::parse(r#"div[class*="JobCard_JobCard__body__"]"#).unwrap();
        let span_selector = Selector::parse("span").unwrap();

        let jobs = document
            .select(&body_selector)
            .filter_map(|body_element| {
                let spans: Vec<_> = body_element.select(&span_selector).collect();

                if spans.len() < 3 {
                    return None;
                }

                let title = spans[0].text().collect::<String>().trim().to_string();
                let company = spans[1].text().collect::<String>().trim().to_string();
                let location_exp = spans[2].text().collect::<String>().trim().to_string();

                let experience_years = if location_exp.contains("경력") {
                    location_exp
                        .split('·')
                        .nth(1)
                        .unwrap_or("N/A")
                        .trim()
                        .to_string()
                } else {
                    "N/A".to_string()
                };

                let url = body_element
                    .parent()
                    .and_then(|parent| parent.value().as_element())
                    .filter(|element| element.name() == "a")
                    .and_then(|element| element.attr("href"))
                    .map(|href| format!("{}{}", self.base_url, href))?;

                if title.is_empty() || company.is_empty() {
                    return None;
                }

                Some(Job::new(title, company, experience_years, url))
            })
            .collect();

        Ok(jobs)
    }
}

impl JobDetailCrawler for WantedClient {
    fn fetch_job_detail(&self, url: &str) -> Result<(Option<String>, Option<String>)> {
        let thread_id = std::thread::current().id();
        let user_agent = get_random_user_agent(thread_id);
        let browser = self.create_browser(user_agent)?;
        self.fetch_job_detail_with_browser(&browser, url)
    }

    fn extract_deadline(&self, html: &str) -> Option<String> {
        let document = Html::parse_document(html);
        let article_selector = Selector::parse(r#"article[class*="JobDueTime"]"#).ok()?;
        let span_selector = Selector::parse("span").ok()?;

        let article = document.select(&article_selector).next()?;
        let span = article.select(&span_selector).next()?;
        let text = span.text().collect::<String>().trim().to_string();

        if text.is_empty() { None } else { Some(text) }
    }

    fn extract_location(&self, html: &str) -> Option<String> {
        let document = Html::parse_document(html);
        let location_selector =
            Selector::parse(r#"div[class*="JobWorkPlace__map__location"]"#).ok()?;

        let location_div = document.select(&location_selector).next()?;
        let text = location_div.text().collect::<String>().trim().to_string();

        if text.is_empty() {
            None
        } else {
            let truncated: String = text.chars().take(16).collect();
            Some(truncated)
        }
    }
}

impl Default for WantedClient {
    fn default() -> Self {
        Self::new(4)
    }
}
