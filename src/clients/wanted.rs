use crate::crawler::{DetailCrawler, JobCrawler, JobFieldExtractor, JobListInfiniteScrollCrawler};
use crate::pipeline::Crawler;
use crate::{Job, Result};
use headless_chrome::Tab;
use scraper::{Html, Selector};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WantedCrawlConfig {
    pub category: WantedJobCategory,
    pub subcategory: WantedJobSubcategory,
    pub total_pages: usize,
    pub min_years: u8,
    pub max_years: u8,
    pub thread_count: usize,
}

impl Default for WantedCrawlConfig {
    fn default() -> Self {
        Self {
            category: WantedJobCategory::Development,
            subcategory: WantedJobSubcategory::Frontend,
            total_pages: 1,
            min_years: 0,
            max_years: 5,
            thread_count: 8,
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
    config: WantedCrawlConfig,
}

impl WantedClient {
    pub fn new(config: WantedCrawlConfig) -> Self {
        Self {
            base_url: "https://www.wanted.co.kr".to_string(),
            config,
        }
    }

    fn build_url(&self) -> String {
        format!(
            "{}/wdlist/{}/{}?country=kr&job_sort=job.recommend_order&years={}&years={}&locations=all",
            self.base_url,
            self.config.category.to_code(),
            self.config.subcategory.to_code(),
            self.config.min_years,
            self.config.max_years
        )
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
    fn parse_html(&self, html: &str) -> Result<Vec<Job>> {
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
        Self::new(WantedCrawlConfig::default())
    }
}

impl Crawler for WantedClient {
    fn start_crawl(&self) -> Result<Vec<Job>> {
        let url = self.build_url();
        let browser = self
            .create_browser()
            .inspect_err(|e| eprintln!("❌ 원티드 채용공고 수집 실패: {}", e))?;

        println!("원티드 채용공고 목록 수집 시작..",);
        self.fetch_all_jobs(&browser, &url, self.config.total_pages)
            .inspect(|jobs| println!("\n✅ 원티드 채용공고 {}개 수집 완료", jobs.len()))
            .inspect_err(|e| eprintln!("❌ 원티드 채용공고 수집 실패: {}", e))
    }
}

impl DetailCrawler for WantedClient {
    fn fetch_job_detail(&self, tab: &Arc<Tab>, job: &Job) -> Result<Job> {
        tab.navigate_to(&job.url)?;
        self.wait_for_detail_page_load(tab)?;

        let html = tab.get_content()?;
        let document = Html::parse_document(&html);
        let deadline = self.extract_deadline(&document);

        std::thread::sleep(Duration::from_millis(500));

        let mut updated_job = job.clone();
        updated_job.deadline = deadline.unwrap_or_default();
        Ok(updated_job)
    }
}
