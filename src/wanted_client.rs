use crate::crawler::{JobCrawler, JobDetailCrawler, JobListCrawler};
use crate::models::{JobCategory, JobSubcategory};
use crate::{Job, Result};
use headless_chrome::Tab;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use scraper::{Html, Selector};
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_MIN_YEARS: u8 = 0;
const DEFAULT_MAX_YEARS: u8 = 5;

pub struct WantedClient {
    base_url: String,
    num_threads: usize,
    category: JobCategory,
    subcategory: JobSubcategory,
    min_years: u8,
    max_years: u8,
}

impl WantedClient {
    pub fn new(num_threads: usize, category: JobCategory, subcategory: JobSubcategory) -> Self {
        Self {
            base_url: "https://www.wanted.co.kr".to_string(),
            num_threads,
            category,
            subcategory,
            min_years: DEFAULT_MIN_YEARS,
            max_years: DEFAULT_MAX_YEARS,
        }
    }

    fn build_url(&self) -> String {
        format!(
            "{}/wdlist/{}/{}?country=kr&job_sort=job.recommend_order&years={}&years={}&locations=all",
            self.base_url,
            self.category as u32,
            self.subcategory as u32,
            self.min_years,
            self.max_years
        )
    }

    pub fn start_crawl(&self, total_pages: usize) -> Result<Vec<Job>> {
        let url = self.build_url();
        let browser = self.create_browser()?;

        println!("프론트엔드 0~5년차 채용공고 목록 수집 시작...");
        let jobs = self.fetch_all_jobs(&browser, &url, total_pages)?;
        let job_counts = jobs.len();
        println!("\n✅ 최종 {}개 채용공고 수집 완료", job_counts);

        println!("\n각 채용공고 상세 수집 시작...");
        let pool = ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .build()?;

        let jobs_with_details: Vec<Job> = pool.install(|| {
            jobs.par_iter()
                .map(|job| {
                    let thread_id = std::thread::current().id();
                    match self.fetch_job_detail(&browser, &job.url) {
                        Ok((deadline, location)) => {
                            println!("[{:?}] 완료: {}", thread_id, job.title);
                            job.clone().with_details(deadline, location)
                        }
                        Err(e) => {
                            eprintln!("[{:?}] 실패 ({}): {}", thread_id, job.title, e);
                            job.clone()
                        }
                    }
                })
                .collect()
        });

        Ok(jobs_with_details)
    }
}

impl JobCrawler for WantedClient {
    fn wait_for_page_load(&self, tab: &Arc<Tab>) -> Result<()> {
        println!("페이지 로드 대기 중...");
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
    fn fetch_job_detail(
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
        Self::new(4, JobCategory::Development, JobSubcategory::Frontend)
    }
}
