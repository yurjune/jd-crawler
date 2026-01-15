use crate::{Job, Result};
use headless_chrome::{Browser, LaunchOptions, Tab};
use rayon::prelude::*;
use scraper::Html;
use std::collections::HashSet;
use std::sync::Arc;

pub trait JobCrawler {
    fn create_browser(&self) -> Result<Browser> {
        let user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        Browser::new(LaunchOptions {
            headless: true,
            args: vec![
                &std::ffi::OsString::from(format!("--user-agent={}", user_agent)),
                &std::ffi::OsString::from("--disable-blink-features=AutomationControlled"),
            ],
            ..Default::default()
        })
        .map_err(Into::into)
    }

    fn wait_for_list_page_load(&self, tab: &Arc<Tab>) -> Result<()>;
    fn wait_for_detail_page_load(&self, tab: &Arc<Tab>) -> Result<()>;
}

pub trait JobListInfiniteScrollCrawler: JobCrawler {
    fn parse_all_jobs(&self, html: &str) -> Result<Vec<Job>>;

    fn go_next_page(&self, tab: &Arc<Tab>) -> Result<()>;

    fn fetch_all_jobs(
        &self,
        browser: &headless_chrome::Browser,
        url: &str,
        total_pages: usize,
    ) -> Result<Vec<Job>> {
        let tab = browser.new_tab()?;
        tab.navigate_to(url)?;
        self.wait_for_list_page_load(&tab)?;

        let mut seen_url = HashSet::new();
        let mut all_jobs = Vec::new();
        let mut no_new_count = 0;

        for current_page in 1..=total_pages {
            let html = tab.get_content()?;
            let new_jobs = self.parse_all_jobs(&html)?;

            let unique_jobs: Vec<_> = new_jobs
                .into_iter()
                .filter(|job| seen_url.insert(job.url.clone()))
                .collect();

            let new_count = unique_jobs.len();
            all_jobs.extend(unique_jobs);

            no_new_count = if new_count == 0 { no_new_count + 1 } else { 0 };

            println!(
                "페이지 {}: 신규 {}개, 총 {}개 수집",
                current_page,
                new_count,
                all_jobs.len()
            );

            if no_new_count >= 2 {
                println!("더 이상 새 데이터 없음 ({}번 연속)", no_new_count);
                break;
            }

            if current_page < total_pages {
                self.go_next_page(&tab)?;
            }
        }

        Ok(all_jobs)
    }
}

pub trait JobListPaginatedCrawler: JobCrawler + Sync {
    fn build_page_url(&self, base_url: &str, page: usize) -> String;

    fn fetch_all_jobs(
        &self,
        browser: &headless_chrome::Browser,
        url: &str,
        total_pages: usize,
        num_threads: usize,
    ) -> Result<Vec<Job>>
    where
        Self: Sync,
    {
        let all_jobs: Vec<Job> = (0..num_threads)
            .into_par_iter()
            .filter_map(|thread_id| {
                let tab = browser.new_tab().ok()?;
                let sys_thread_id = std::thread::current().id();

                let pages: Vec<usize> =
                    (thread_id + 1..=total_pages).step_by(num_threads).collect();

                let jobs: Vec<Job> = pages
                    .iter()
                    .filter_map(|&page_num| {
                        let url = self.build_page_url(url, page_num);
                        tab.navigate_to(&url).ok()?;
                        self.wait_for_list_page_load(&tab).ok()?;

                        let html = tab.get_content().ok()?;
                        let page_jobs = self.parse_job(&html).ok()?;

                        println!("[{:?}] 페이지 {} 완료", sys_thread_id, page_num);
                        Some(page_jobs)
                    })
                    .flatten()
                    .collect();

                Some(jobs)
            })
            .flatten()
            .collect();

        Ok(all_jobs)
    }

    fn parse_job(&self, html: &str) -> Result<Vec<Job>>;
}

pub trait JobFieldExtractor {
    fn extract_title(&self, fragment: &Html) -> Option<String>;
    fn extract_company(&self, fragment: &Html) -> Option<String>;
    fn extract_experience_years(&self, fragment: &Html) -> Option<String>;
    fn extract_url(&self, fragment: &Html) -> Option<String>;
    fn extract_deadline(&self, fragment: &Html) -> Option<String>;
    fn extract_location(&self, fragment: &Html) -> Option<String>;
}
