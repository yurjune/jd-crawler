use headless_chrome::Tab;
use scraper::{Html, Selector};
use std::sync::Arc;
use std::time::Duration;

use crate::JobCrawler;
use crate::enricher::{EnricherConfig, JobEnricher};
use crate::{Job, Result};
use regex::Regex;

pub struct BlindEnricher {
    base_url: String,
    config: EnricherConfig,
}

impl BlindEnricher {
    pub fn new(config: EnricherConfig) -> Self {
        Self {
            base_url: "https://www.teamblind.com/kr/company".to_string(),
            config,
        }
    }
}

impl JobEnricher for BlindEnricher {
    fn start_enrich(&self, jobs: &[Job]) -> Result<Vec<Job>> {
        println!("\n블라인드 평점/리뷰 개수 수집 시작..");

        let browser = self
            .create_browser()
            .inspect_err(|e| eprintln!("❌ 블라인드 평점/리뷰 개수 수집 실패: {}", e))?;

        self.enrich_all_jobs(&browser, jobs, self.config.thread_count)
            .inspect(|_| println!("✅ 블라인드 평점/리뷰 개수 수집 완료"))
            .inspect_err(|e| eprintln!("❌ 블라인드 평점/리뷰 개수 수집 실패: {}", e))
    }

    fn build_url(&self, company: &str) -> String {
        format!("{}/{}/reviews", self.base_url, company)
    }

    fn fetch_rate_and_reviews(
        &self,
        tab: &Arc<Tab>,
        url: &str,
    ) -> Result<(Option<String>, Option<u32>)> {
        tab.navigate_to(url)?;

        let html = tab.get_content()?;
        std::thread::sleep(Duration::from_millis(500));
        self.parse_data(&html)
    }

    fn parse_data(&self, html: &str) -> Result<(Option<String>, Option<u32>)> {
        let document = Html::parse_document(html);
        let rating = self.extract_rating(&document);
        let review_count = self.extract_review_count(&document);
        Ok((rating, review_count))
    }

    fn extract_rating(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse("script[type='application/ld+json']").ok()?;
        let re = Regex::new(r#""ratingValue":"([^"]+)""#).ok()?;

        for script in document.select(&selector) {
            let json_text = script.text().collect::<String>();

            if json_text.contains("EmployerAggregateRating") {
                if let Some(captures) = re.captures(&json_text) {
                    return Some(captures.get(1)?.as_str().to_string());
                }
            }
        }

        None
    }

    fn extract_review_count(&self, document: &Html) -> Option<u32> {
        let selector = Selector::parse("script[type='application/ld+json']").ok()?;
        let re = Regex::new(r#""ratingCount":(\d+)"#).ok()?;

        for script in document.select(&selector) {
            let json_text = script.text().collect::<String>();

            if json_text.contains("EmployerAggregateRating") {
                if let Some(captures) = re.captures(&json_text) {
                    let number_str = captures.get(1)?.as_str();
                    return number_str.parse::<u32>().ok();
                }
            }
        }

        None
    }
}

impl JobCrawler for BlindEnricher {}
