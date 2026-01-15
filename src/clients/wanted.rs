use crate::crawler::{JobCrawler, JobFieldExtractor, JobListInfiniteScrollCrawler};
use crate::{Job, Result};
use headless_chrome::Tab;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use scraper::{Html, Selector};
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
}

impl Default for WantedCrawlConfig {
    fn default() -> Self {
        Self {
            total_pages: 1,
            num_threads: 4,
            min_years: 0,
            max_years: 5,
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

        println!("\n각 채용공고 상세 수집 시작...");
        let pool = ThreadPoolBuilder::new()
            .num_threads(config.num_threads)
            .build()?;

        let jobs_with_details: Vec<Job> = pool.install(|| {
            jobs.par_iter()
                .map(|job| {
                    let thread_id = std::thread::current().id();
                    let mut job_clone = job.clone();

                    match self.fetch_job_detail(&browser, &job.url) {
                        Ok(deadline) => {
                            println!("[{:?}] 완료: {}", thread_id, job.title);
                            job_clone.deadline = deadline;
                        }
                        Err(e) => {
                            eprintln!("[{:?}] 실패 ({}): {}", thread_id, job.title, e);
                        }
                    }

                    job_clone
                })
                .collect()
        });

        Ok(jobs_with_details)
    }

    fn fetch_job_detail(
        &self,
        browser: &headless_chrome::Browser,
        url: &str,
    ) -> Result<Option<String>> {
        let tab = browser.new_tab()?;

        tab.navigate_to(url)?;
        tab.wait_for_element("body")?;
        std::thread::sleep(Duration::from_secs(2));

        let html = tab.get_content()?;
        let document = Html::parse_document(&html);

        let deadline = self.extract_deadline(&document);

        std::thread::sleep(Duration::from_secs(1));

        Ok(deadline)
    }
}

impl JobCrawler for WantedClient {
    fn wait_for_page_load(&self, tab: &Arc<Tab>) -> Result<()> {
        tab.wait_for_element(r#"[data-cy="job-list"]"#)?;
        Ok(())
    }
}

impl JobListInfiniteScrollCrawler for WantedClient {
    fn parse_all_jobs(&self, html: &str) -> Result<Vec<Job>> {
        let document = Html::parse_document(html);

        let body_selector = Selector::parse(r#"div[class*="JobCard_JobCard__body__"]"#).unwrap();

        let jobs = document
            .select(&body_selector)
            .filter_map(|body_element| {
                let body_html = body_element.html();
                let body_doc = Html::parse_fragment(&body_html);

                let title = self.extract_title(&body_doc)?;
                let company = self.extract_company(&body_doc)?;
                let experience_years = self.extract_experience_years(&body_doc)?;

                let url = body_element
                    .parent()
                    .and_then(|parent| parent.value().as_element())
                    .filter(|element| element.name() == "a")
                    .and_then(|element| element.attr("href"))
                    .map(|href| format!("{}{}", self.base_url, href))?;

                Some(Job::new(title, company, experience_years, url))
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

        // "서울 강남구∙경력 3년 이상" 형태에서 경력 부분만 추출
        match location_exp.split('∙').nth(1) {
            Some(experience) => Some(experience.trim().to_string()),
            None => Some(location_exp),
        }
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

        // "서울 강남구∙경력 3년 이상" 형태에서 위치 부분만 추출
        match location_exp.split('∙').next() {
            Some(location) => Some(location.trim().to_string()),
            None => Some(location_exp),
        }
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
