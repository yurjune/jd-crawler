use crate::{Job, Result};
use headless_chrome::{Browser, LaunchOptions};
use scraper::{Html, Selector};
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

    pub fn fetch_frontend_jobs(&self, min_years: u8, max_years: u8) -> Result<Vec<Job>> {
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

        // Wait for job list container to appear
        println!("페이지 로딩 대기 중...");
        tab.wait_for_element(r#"[data-cy="job-list"]"#)?;

        // Extra wait for dynamic content
        std::thread::sleep(Duration::from_secs(3));

        let html = tab.get_content()?;

        // Parse HTML and extract jobs
        let jobs = self.parse_jobs(&html)?;
        Ok(jobs)
    }

    fn parse_jobs(&self, html: &str) -> Result<Vec<Job>> {
        let document = Html::parse_document(html);

        // Select all job card body sections
        let body_selector = Selector::parse(r#"div[class*="JobCard_JobCard__body__"]"#).unwrap();
        let span_selector = Selector::parse("span").unwrap();

        let jobs = document
            .select(&body_selector)
            .filter_map(|body_element| {
                // Get all span elements inside the body
                let spans: Vec<_> = body_element.select(&span_selector).collect();

                // should be
                // 웹 프론트엔드 개발자
                // 현대 오토에버
                // 서울 강남구∙경력 4년 이상
                if spans.len() < 3 {
                    return None;
                }

                let title = spans[0].text().collect::<String>().trim().to_string();
                let company = spans[1].text().collect::<String>().trim().to_string();
                let location_exp = spans[2].text().collect::<String>().trim().to_string();

                let experience_years = if location_exp.contains("경력") {
                    location_exp
                        .split('·')
                        .nth(1)
                        .unwrap_or("N/A")
                        .trim()
                        .to_string()
                } else {
                    "N/A".to_string()
                };

                // Find the parent <a> tag to get the URL
                let url = body_element
                    .parent()
                    .and_then(|parent| parent.value().as_element())
                    .filter(|element| element.name() == "a")
                    .and_then(|element| element.attr("href"))
                    .map(|href| format!("{}{}", self.base_url, href))?;

                // Skip if missing required fields
                if title.is_empty() || company.is_empty() {
                    return None;
                }

                Some(Job::new(title, company, experience_years, url))
            })
            .collect();

        Ok(jobs)
    }
}

impl Default for WantedClient {
    fn default() -> Self {
        Self::new()
    }
}
