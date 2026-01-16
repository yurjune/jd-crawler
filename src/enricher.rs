use crate::{Job, Result};
use headless_chrome::Tab;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use regex::Regex;
use scraper::Html;
use std::collections::HashMap;
use std::sync::Arc;

pub struct EnricherConfig {
    pub thread_count: usize,
}

pub trait JobEnricher: Sync {
    fn start_enrich(&self, jobs: Vec<Job>) -> Result<Vec<Job>>;

    fn enrich_all_jobs(
        &self,
        browser: &headless_chrome::Browser,
        jobs: Vec<Job>,
        thread_count: usize,
    ) -> Result<Vec<Job>> {
        let mut tabs_map = HashMap::new();
        for i in 0..thread_count {
            tabs_map.insert(i, browser.new_tab()?);
        }
        let tabs = tabs_map;
        let pool = ThreadPoolBuilder::new().num_threads(thread_count).build()?;

        let enriched_jobs = pool.install(|| {
            jobs.into_par_iter()
                .map(|mut job| {
                    let thread_idx = rayon::current_thread_index().unwrap();
                    let tab = &tabs[&thread_idx];

                    let normalized_company = self.normalize_company_name(&job.company);
                    let url = self.build_url(&normalized_company);

                    match self.fetch_rate_and_reviews(tab, &url) {
                        Ok((rating, review_count)) => {
                            println!("[Thread {:?}] 완료: {}", thread_idx, job.company);
                            job.rating = rating;
                            job.review_count = review_count;
                        }
                        Err(e) => {
                            eprintln!("[Thread {:?}] 실패 ({}): {}", thread_idx, job.company, e);
                        }
                    }

                    job
                })
                .collect()
        });

        Ok(enriched_jobs)
    }

    fn fetch_rate_and_reviews(
        &self,
        tab: &Arc<Tab>,
        url: &str,
    ) -> Result<(Option<String>, Option<u32>)>;

    fn build_url(&self, company: &str) -> String;

    fn normalize_company_name(&self, company: &str) -> String {
        let re = Regex::new(r"\s*[(\(（][^)\)）]*[)\)）]\s*").unwrap();
        re.replace_all(company, "").trim().to_string()
    }

    fn parse_data(&self, html: &str) -> Result<(Option<String>, Option<u32>)>;

    fn extract_rating(&self, document: &Html) -> Option<String>;

    fn extract_review_count(&self, document: &Html) -> Option<u32>;
}
