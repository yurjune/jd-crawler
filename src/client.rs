use crate::Result;
use headless_chrome::{Browser, LaunchOptions};
use std::time::Duration;

const JOB_CATEGORY_DEVELOPMENT: u32 = 518;
const JOB_SUBCATEGORY_FRONTEND: u32 = 669;

pub struct WantedClient {
    base_url: String,
}

impl WantedClient {
    pub fn new() -> Self {
        Self {
            base_url: "https://www.wanted.co.kr".to_string(),
        }
    }

    pub fn fetch_frontend_jobs(&self, min_years: u8, max_years: u8) -> Result<String> {
        let url = format!(
            "{}/wdlist/{}/{}?country=kr&job_sort=job.recommend_order&years={}&years={}&locations=all",
            self.base_url, JOB_CATEGORY_DEVELOPMENT, JOB_SUBCATEGORY_FRONTEND, min_years, max_years
        );

        println!("요청 URL: {}", url);

        // Pretend like a real user to avoid bot detection
        let user_agent = std::ffi::OsString::from(
            "--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        );
        // Hide webdriver flag to avoid bot detection
        let disable_auto =
            std::ffi::OsString::from("--disable-blink-features=AutomationControlled");

        let browser = Browser::new(LaunchOptions {
            headless: true,
            args: vec![&user_agent, &disable_auto],
            ..Default::default()
        })?;

        let tab = browser.new_tab()?;

        tab.navigate_to(&url)?;
        tab.wait_for_element("body")?;

        // Wait for full HTML build of SPA
        std::thread::sleep(Duration::from_secs(5));

        let html = tab.get_content()?;
        Ok(html)
    }
}

impl Default for WantedClient {
    fn default() -> Self {
        Self::new()
    }
}
