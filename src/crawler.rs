use crate::{Job, Result};
use headless_chrome::{Browser, LaunchOptions, Tab};
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use scraper::Html;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

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

    fn wait_for_list_page_load(&self, _tab: &Arc<Tab>) -> Result<()> {
        Ok(())
    }

    fn wait_for_detail_page_load(&self, _tab: &Arc<Tab>) -> Result<()> {
        Ok(())
    }
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

    fn fetch_jobs(&self, tab: &Arc<Tab>, url: &str) -> Result<Vec<Job>> {
        tab.navigate_to(url)?;
        self.wait_for_list_page_load(tab)?;
        let html = tab.get_content()?;
        std::thread::sleep(Duration::from_millis(500));
        self.parse_job(&html)
    }

    fn fetch_all_jobs(
        &self,
        browser: &headless_chrome::Browser,
        url: &str,
        total_pages: usize,
        num_threads: usize,
    ) -> Result<Vec<Job>> {
        let pool = ThreadPoolBuilder::new().num_threads(num_threads).build()?;

        let mut tabs_map = HashMap::new();
        for i in 0..num_threads {
            tabs_map.insert(i, browser.new_tab()?);
        }
        let tabs = tabs_map;

        let all_jobs: Vec<Job> = pool.install(|| {
            (1..=total_pages)
                .into_par_iter()
                .flat_map(|page| {
                    let thread_index = rayon::current_thread_index().unwrap();
                    let tab = &tabs[&thread_index];
                    let url = self.build_page_url(url, page);
                    let result = self.fetch_jobs(tab, &url);

                    match result {
                        Ok(page_jobs) => {
                            println!("[Thread {:?}] 완료: 페이지 {}", thread_index, page);
                            page_jobs
                        }
                        Err(e) => {
                            eprintln!("[Thread {:?}] 실패 (페이지 {}): {}", thread_index, page, e);
                            Vec::new()
                        }
                    }
                })
                .collect()
        });

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
