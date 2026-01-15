use crate::Result;
use crate::crawler::{JobCrawler, JobListPaginatedCrawler};
use crate::models::{CrawlConfig, Job};
use headless_chrome::Tab;
use scraper::{Html, Selector};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum SaraminJobCategory {
    Frontend,
}

impl SaraminJobCategory {
    pub fn to_word(&self) -> &str {
        match self {
            Self::Frontend => "프론트엔드",
        }
    }
}

pub struct SaraminClient {
    base_url: String,
    category: SaraminJobCategory,
}

impl SaraminClient {
    pub fn new(category: SaraminJobCategory) -> Self {
        Self {
            base_url: "https://www.saramin.co.kr/zf_user/search/recruit".to_string(),
            category,
        }
    }

    pub fn start_crawl(&self, config: CrawlConfig) -> Result<Vec<Job>> {
        let browser = self.create_browser()?;
        let refined_url = format!("{}?searchword={}", self.base_url, self.category.to_word());

        println!("사람인 채용공고 목록 수집 시작..",);
        let jobs = self.fetch_all_jobs(
            &browser,
            &refined_url,
            config.total_pages,
            config.num_threads,
        )?;
        println!("총 {}개 수집 완료", jobs.len());
        Ok(jobs)
    }
}

impl JobCrawler for SaraminClient {
    fn wait_for_page_load(&self, tab: &Arc<Tab>) -> Result<()> {
        tab.wait_for_element(r"#recruit_info_list")?;
        Ok(())
    }
}

impl JobListPaginatedCrawler for SaraminClient {
    fn build_page_url(&self, base_url: &str, page: usize) -> String {
        format!("{}&recruitPage={}", base_url, page)
    }

    fn parse_job(&self, html: &str) -> Result<Vec<Job>> {
        let document = Html::parse_document(html);
        let job_card_selector = Selector::parse(r"div.item_recruit").unwrap();
        let job_title_selector = Selector::parse(r"h2.job_tit").unwrap();
        let job_date_selector = Selector::parse(r"span.date").unwrap();
        let job_condition_selector = Selector::parse(r"div.job_condition").unwrap();
        let company_selector = Selector::parse(r"strong.corp_name").unwrap();
        let span_selector = Selector::parse("span").unwrap();
        let anchor_selector = Selector::parse("a").unwrap();

        let jobs = document
            .select(&job_card_selector)
            .filter_map(|card| {
                let title_el = card.select(&job_title_selector).next()?;
                let link = title_el.select(&anchor_selector).next()?;
                let title = link.value().attr("title")?.to_string();
                let href = link.value().attr("href")?;
                let url = format!("https://www.saramin.co.kr{}", href);

                let condition = card.select(&job_condition_selector).next()?;
                let spans: Vec<_> = condition.select(&span_selector).collect();

                let experience_years = spans
                    .get(1)
                    .map(|span| span.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let location = spans
                    .first()
                    .map(|span| {
                        span.select(&anchor_selector)
                            .map(|a| a.text().collect::<String>().trim().to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .unwrap_or_default();

                let deadline = card
                    .select(&job_date_selector)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let company = card
                    .select(&company_selector)
                    .next()
                    .and_then(|strong| strong.select(&anchor_selector).next())
                    .map(|a| a.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let mut job = Job::new(title, company, experience_years, url);
                job.deadline = if deadline.is_empty() {
                    None
                } else {
                    Some(deadline)
                };
                job.location = if location.is_empty() {
                    None
                } else {
                    Some(location)
                };

                Some(job)
            })
            .collect();

        Ok(jobs)
    }
}
