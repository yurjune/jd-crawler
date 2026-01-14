use crate::{Job, Result};
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

pub trait JobCrawler {
    fn fetch_all_jobs(&self, url: &str, max_scroll: usize) -> Result<Vec<Job>> {
        println!("요청 URL: {}", url);

        let browser = self.create_browser("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")?;
        let tab = browser.new_tab()?;
        tab.navigate_to(url)?;

        self.wait_for_page_load(&tab)?;

        let jobs = self.collect_all_jobs_with_scroll(&tab, max_scroll)?;
        println!("\n✅ 최종 {}개 채용공고 수집 완료", jobs.len());

        Ok(jobs)
    }

    fn create_browser(&self, user_agent: &str) -> Result<Browser> {
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

    fn wait_for_page_load(&self, tab: &Arc<Tab>) -> Result<()>;

    fn parse_jobs(&self, html: &str) -> Result<Vec<Job>>;

    fn fetch_job_detail(&self, url: &str) -> Result<(Option<String>, Option<String>)>;

    fn extract_deadline(&self, html: &str) -> Option<String>;

    fn extract_location(&self, html: &str) -> Option<String>;

    fn scroll_down(&self, tab: &Arc<Tab>) -> Result<()> {
        tab.evaluate("window.scrollTo(0, document.body.scrollHeight)", false)?;
        std::thread::sleep(Duration::from_secs(2));
        Ok(())
    }

    fn collect_all_jobs_with_scroll(&self, tab: &Arc<Tab>, max_scroll: usize) -> Result<Vec<Job>> {
        let mut seen_url = HashSet::new();
        let mut all_jobs = Vec::new();
        let mut no_new_count = 0;

        for scroll_count in 0..=max_scroll {
            let html = tab.get_content()?;
            let new_jobs = self.parse_jobs(&html)?;

            let unique_jobs: Vec<_> = new_jobs
                .into_iter()
                .filter(|job| seen_url.insert(job.url.clone()))
                .collect();

            let new_count = unique_jobs.len();
            all_jobs.extend(unique_jobs);

            no_new_count = if new_count == 0 { no_new_count + 1 } else { 0 };

            println!(
                "스크롤 {}: 신규 {}개, 총 {}개 수집",
                scroll_count + 1,
                new_count,
                all_jobs.len()
            );

            if no_new_count >= 2 {
                println!("더 이상 새 데이터 없음 ({}번 연속)", no_new_count);
                break;
            }

            if scroll_count < max_scroll {
                self.scroll_down(tab)?;
            }
        }

        Ok(all_jobs)
    }
}
