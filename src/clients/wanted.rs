use crate::crawler::{JobCrawler, JobFieldExtractor, JobListInfiniteScrollCrawler};
use crate::{Job, Result};
use headless_chrome::Tab;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WantedCrawlConfig {
    /// 크롤링할 페이지 수
    pub total_pages: usize,
    /// 병렬 처리에 사용할 스레드 개수
    pub num_threads: usize,
    /// 최소 경력 (년)
    pub min_years: u8,
    /// 최대 경력 (년)
    pub max_years: u8,
    /// 각 상세 페이지 추가 크롤링
    pub full_crawl: bool,
}

impl Default for WantedCrawlConfig {
    fn default() -> Self {
        Self {
            total_pages: 1,
            num_threads: 4,
            min_years: 0,
            max_years: 5,
            full_crawl: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WantedJobCategory {
    Development,
}

impl WantedJobCategory {
    pub fn to_code(&self) -> u32 {
        match self {
            Self::Development => 518,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WantedJobSubcategory {
    Frontend,
    Backend,
    Web,
    Android,
    IOS,
}

impl WantedJobSubcategory {
    pub fn to_code(&self) -> u32 {
        match self {
            Self::Frontend => 669,
            Self::Backend => 872,
            Self::Web => 873,
            Self::Android => 677,
            Self::IOS => 678,
        }
    }
}

pub struct WantedClient {
    base_url: String,
    category: WantedJobCategory,
    subcategory: WantedJobSubcategory,
}

impl WantedClient {
    pub fn new(category: WantedJobCategory, subcategory: WantedJobSubcategory) -> Self {
        Self {
            base_url: "https://www.wanted.co.kr".to_string(),
            category,
            subcategory,
        }
    }

    fn build_url(&self, config: &WantedCrawlConfig) -> String {
        format!(
            "{}/wdlist/{}/{}?country=kr&job_sort=job.recommend_order&years={}&years={}&locations=all",
            self.base_url,
            self.category.to_code(),
            self.subcategory.to_code(),
            config.min_years,
            config.max_years
        )
    }

    pub fn start_crawl(&self, config: WantedCrawlConfig) -> Result<Vec<Job>> {
        let url = self.build_url(&config);
        let browser = self.create_browser()?;

        println!("원티드 채용공고 목록 수집 시작..",);
        let jobs = self.fetch_all_jobs(&browser, &url, config.total_pages)?;
        let job_counts = jobs.len();
        println!("\n✅ 최종 {}개 채용공고 수집 완료", job_counts);

        if !config.full_crawl {
            return Ok(jobs);
        }

        println!("\n각 채용공고 상세 수집 시작...");
        let jobs_with_details = self.fetch_all_job_detail(&browser, jobs, config.num_threads)?;

        Ok(jobs_with_details)
    }

    fn fetch_all_job_detail(
        &self,
        browser: &headless_chrome::Browser,
        jobs: Vec<Job>,
        num_threads: usize,
    ) -> Result<Vec<Job>> {
        let pool = ThreadPoolBuilder::new().num_threads(num_threads).build()?;

        let mut tabs_map = HashMap::new();
        for i in 0..num_threads {
            tabs_map.insert(i, browser.new_tab()?);
        }
        let tabs = tabs_map;

        let jobs_with_details: Vec<Job> = pool.install(|| {
            jobs.into_par_iter()
                .map(|mut job| {
                    let thread_index = rayon::current_thread_index().unwrap();
                    let tab = &tabs[&thread_index];

                    match self.fetch_job_detail(tab, &job.url) {
                        Ok(deadline) => {
                            println!("[Thread {:?}] 완료: {}", thread_index, job.title);
                            job.deadline = deadline.unwrap_or_default();
                        }
                        Err(e) => {
                            eprintln!("[Thread {:?}] 실패 ({}): {}", thread_index, job.title, e);
                        }
                    }
                    job
                })
                .collect()
        });

        Ok(jobs_with_details)
    }

    fn fetch_job_detail(
        &self,
        tab: &Arc<headless_chrome::Tab>,
        url: &str,
    ) -> Result<Option<String>> {
        tab.navigate_to(url)?;
        self.wait_for_detail_page_load(tab)?;
        std::thread::sleep(Duration::from_secs(2));

        let html = tab.get_content()?;
        let document = Html::parse_document(&html);

        let deadline = self.extract_deadline(&document);

        std::thread::sleep(Duration::from_secs(1));

        Ok(deadline)
    }
}

impl JobCrawler for WantedClient {
    fn wait_for_list_page_load(&self, tab: &Arc<Tab>) -> Result<()> {
        tab.wait_for_element(r#"div[class*="JobCard_JobCard__body__"]"#)?;
        Ok(())
    }

    fn wait_for_detail_page_load(&self, tab: &Arc<Tab>) -> Result<()> {
        tab.wait_for_element("body")?;
        Ok(())
    }
}

impl JobListInfiniteScrollCrawler for WantedClient {
    fn parse_all_jobs(&self, html: &str) -> Result<Vec<Job>> {
        let document = Html::parse_document(html);

        let body_selector = Selector::parse(r#"div[class*="JobCard_JobCard__body__"]"#).unwrap();

        let jobs = document
            .select(&body_selector)
            .map(|body_element| {
                let body_doc = Html::parse_fragment(&body_element.html());

                let title = self.extract_title(&body_doc).unwrap_or_default();
                let company = self.extract_company(&body_doc).unwrap_or_default();
                let experience_years = self.extract_experience_years(&body_doc).unwrap_or_default();
                let location = self.extract_location(&body_doc).unwrap_or_default();

                let url = body_element
                    .parent()
                    .and_then(|parent| parent.value().as_element())
                    .filter(|element| element.name() == "a")
                    .and_then(|element| element.attr("href"))
                    .map(|href| format!("{}{}", self.base_url, href))
                    .unwrap_or_default();

                Job {
                    title,
                    company,
                    experience_years,
                    url,
                    location,
                    ..Default::default()
                }
            })
            .collect();

        Ok(jobs)
    }

    fn go_next_page(&self, tab: &Arc<Tab>) -> Result<()> {
        tab.evaluate("window.scrollTo(0, document.body.scrollHeight)", false)?;
        std::thread::sleep(Duration::from_secs(2));
        Ok(())
    }
}

impl JobFieldExtractor for WantedClient {
    fn extract_title(&self, fragment: &Html) -> Option<String> {
        let selector = Selector::parse("span").ok()?;
        let spans: Vec<_> = fragment.select(&selector).collect();
        let text = spans.first()?.text().collect::<String>().trim().to_string();
        Some(text)
    }

    fn extract_company(&self, fragment: &Html) -> Option<String> {
        let selector = Selector::parse("span").ok()?;
        let spans: Vec<_> = fragment.select(&selector).collect();
        let text = spans.get(1)?.text().collect::<String>().trim().to_string();
        Some(text)
    }

    fn extract_experience_years(&self, fragment: &Html) -> Option<String> {
        let selector = Selector::parse("span").ok()?;
        let spans: Vec<_> = fragment.select(&selector).collect();
        let location_exp = spans.get(2)?.text().collect::<String>().trim().to_string();

        for sep in ['∙', '·', '•', '/', '|'] {
            if let Some(experience) = location_exp.split(sep).nth(1) {
                return Some(experience.trim().to_string());
            }
        }

        Some(location_exp)
    }

    fn extract_url(&self, _fragment: &Html) -> Option<String> {
        // URL은 JobCard body의 부모 요소에서 추출하므로 여기서는 구현하지 않음
        None
    }

    fn extract_deadline(&self, fragment: &Html) -> Option<String> {
        let article_selector = Selector::parse(r#"article[class*="JobDueTime"]"#).ok()?;
        let span_selector = Selector::parse("span").ok()?;
        let article = fragment.select(&article_selector).next()?;
        let span = article.select(&span_selector).next()?;
        let text = span.text().collect::<String>().trim().to_string();
        Some(text)
    }

    fn extract_location(&self, fragment: &Html) -> Option<String> {
        let selector = Selector::parse("span").ok()?;
        let spans: Vec<_> = fragment.select(&selector).collect();
        let location_exp = spans.get(2)?.text().collect::<String>().trim().to_string();

        for sep in ['∙', '·', '•', '/', '|'] {
            if location_exp.contains(sep) {
                return location_exp.split(sep).next().map(|s| s.trim().to_string());
            }
        }

        Some(location_exp)
    }
}

impl Default for WantedClient {
    fn default() -> Self {
        Self::new(
            WantedJobCategory::Development,
            WantedJobSubcategory::Frontend,
        )
    }
}
