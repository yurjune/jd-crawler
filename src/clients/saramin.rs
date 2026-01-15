use crate::Result;
use crate::crawler::{JobCrawler, JobFieldExtractor, JobListPaginatedCrawler};
use crate::models::Job;
use headless_chrome::Tab;
use scraper::{Html, Selector};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SaraminCrawlConfig {
    /// 크롤링할 페이지 수
    pub total_pages: usize,
    /// 병렬 처리에 사용할 스레드 개수
    pub num_threads: usize,
}

impl Default for SaraminCrawlConfig {
    fn default() -> Self {
        Self {
            total_pages: 1,
            num_threads: 4,
        }
    }
}

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

    pub fn start_crawl(&self, config: SaraminCrawlConfig) -> Result<Vec<Job>> {
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

        let jobs = document
            .select(&job_card_selector)
            .filter_map(|card| {
                let card_html = card.html();
                let card_fragment = Html::parse_fragment(&card_html);

                let title = self.extract_title(&card_fragment)?;
                let company = self.extract_company(&card_fragment)?;
                let experience_years = self.extract_experience_years(&card_fragment)?;
                let url = self.extract_url(&card_fragment)?;
                let deadline = self.extract_deadline(&card_fragment);
                let location = self.extract_location(&card_fragment);

                let mut job = Job::new(title, company, experience_years, url);
                job.deadline = deadline;
                job.location = location;

                Some(job)
            })
            .collect();

        Ok(jobs)
    }
}

impl JobFieldExtractor for SaraminClient {
    fn extract_title(&self, fragment: &Html) -> Option<String> {
        let job_title_selector = Selector::parse(r"h2.job_tit").ok()?;
        let anchor_selector = Selector::parse("a").ok()?;

        let title_el = fragment.select(&job_title_selector).next()?;
        let link = title_el.select(&anchor_selector).next()?;
        let title = link.value().attr("title")?.to_string();
        Some(title)
    }

    fn extract_company(&self, fragment: &Html) -> Option<String> {
        let company_selector = Selector::parse(r"strong.corp_name").ok()?;
        let anchor_selector = Selector::parse("a").ok()?;

        let text = fragment
            .select(&company_selector)
            .next()?
            .select(&anchor_selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        Some(text)
    }

    fn extract_experience_years(&self, fragment: &Html) -> Option<String> {
        let job_condition_selector = Selector::parse(r"div.job_condition").ok()?;
        let span_selector = Selector::parse("span").ok()?;

        let condition = fragment.select(&job_condition_selector).next()?;
        let spans: Vec<_> = condition.select(&span_selector).collect();
        let text = spans.get(1)?.text().collect::<String>().trim().to_string();
        Some(text)
    }

    fn extract_url(&self, fragment: &Html) -> Option<String> {
        let job_title_selector = Selector::parse(r"h2.job_tit").ok()?;
        let anchor_selector = Selector::parse("a").ok()?;

        let title_el = fragment.select(&job_title_selector).next()?;
        let link = title_el.select(&anchor_selector).next()?;
        let href = link.value().attr("href")?;
        let url = format!("https://www.saramin.co.kr{}", href);
        Some(url)
    }

    fn extract_deadline(&self, fragment: &Html) -> Option<String> {
        let job_date_selector = Selector::parse(r"span.date").ok()?;
        let text = fragment
            .select(&job_date_selector)
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        Some(text)
    }

    fn extract_location(&self, fragment: &Html) -> Option<String> {
        let job_condition_selector = Selector::parse(r"div.job_condition").ok()?;
        let span_selector = Selector::parse("span").ok()?;
        let anchor_selector = Selector::parse("a").ok()?;

        let condition = fragment.select(&job_condition_selector).next()?;
        let spans: Vec<_> = condition.select(&span_selector).collect();
        let text = spans
            .first()?
            .select(&anchor_selector)
            .map(|a| a.text().collect::<String>().trim().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        Some(text)
    }
}
