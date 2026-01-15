use headless_chrome::Tab;
use scraper::{Html, Selector};
use std::sync::Arc;

use crate::JobCrawler;
use crate::enrichers::job_enricher::JobEnricher;
use crate::{Job, Result};
use regex::Regex;

pub struct BlindEnricher {
    base_url: String,
}

impl BlindEnricher {
    pub fn new() -> Self {
        Self {
            base_url: "https://www.teamblind.com/kr/company".to_string(),
        }
    }

    pub fn start_enrich(&self, jobs: Vec<Job>, thread_count: usize) -> Result<Vec<Job>> {
        let browser = self.create_browser()?;
        println!("\n블라인드 평점/리뷰 수집 시작..");
        let enriched_jobs = self.enrich(&browser, jobs, thread_count)?;
        println!("\n✅ 블라인드 enrichment 완료: {}개", enriched_jobs.len());
        Ok(enriched_jobs)
    }
}

impl JobEnricher for BlindEnricher {
    fn build_url(&self, company: &str) -> String {
        format!("{}/{}/reviews", self.base_url, company)
    }

    fn fetch_rate_and_reviews(
        &self,
        tab: &Arc<Tab>,
        url: &str,
    ) -> Result<(Option<String>, Option<u32>)> {
        tab.navigate_to(url)?;
        std::thread::sleep(std::time::Duration::from_secs(2));

        let html = tab.get_content()?;
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

impl Default for BlindEnricher {
    fn default() -> Self {
        Self::new()
    }
}

impl JobCrawler for BlindEnricher {}
