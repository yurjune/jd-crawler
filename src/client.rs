use crate::{Job, Result};
use headless_chrome::{Browser, LaunchOptions, Tab};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
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
        let url = self.build_url(min_years, max_years);
        println!("요청 URL: {}", url);

        let browser = self.create_browser()?;
        let tab = browser.new_tab()?;
        tab.navigate_to(&url)?;

        self.wait_for_page_load(&tab)?;

        let jobs = self.collect_all_jobs_with_scroll(&tab)?;
        println!("\n✅ 최종 {}개 채용공고 수집 완료", jobs.len());

        Ok(jobs)
    }

    fn build_url(&self, min_years: u8, max_years: u8) -> String {
        format!(
            "{}/wdlist/{}/{}?country=kr&job_sort=job.recommend_order&years={}&years={}&locations=all",
            self.base_url, JOB_CATEGORY_DEVELOPMENT, JOB_SUBCATEGORY_FRONTEND, min_years, max_years
        )
    }

    fn create_browser(&self) -> Result<Browser> {
        Browser::new(LaunchOptions {
            headless: true,
            args: vec![
                &std::ffi::OsString::from("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
                &std::ffi::OsString::from("--disable-blink-features=AutomationControlled"),
            ],
            ..Default::default()
        })
        .map_err(Into::into)
    }

    fn wait_for_page_load(&self, tab: &Arc<Tab>) -> Result<()> {
        println!("페이지 로딩 대기 중...");
        tab.wait_for_element("body")?;
        tab.wait_for_element(r#"[data-cy="job-list"]"#)?;
        std::thread::sleep(Duration::from_secs(3));
        Ok(())
    }

    fn collect_all_jobs_with_scroll(&self, tab: &Arc<Tab>) -> Result<Vec<Job>> {
        let mut seen = HashSet::new();
        let mut all_jobs = Vec::new();
        let mut no_new_count = 0;

        for scroll_count in 0..50 {
            let new_jobs = self.fetch_and_scroll(tab, scroll_count)?;
            let unique_jobs: Vec<_> = new_jobs
                .into_iter()
                .filter(|job| seen.insert(job.url.clone()))
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
        }

        Ok(all_jobs)
    }

    fn fetch_and_scroll(&self, tab: &Arc<Tab>, scroll_count: usize) -> Result<Vec<Job>> {
        let html = tab.get_content()?;
        let jobs = self.parse_jobs(&html)?;

        if scroll_count < 49 {
            self.scroll_down(tab)?;
        }

        Ok(jobs)
    }

    fn scroll_down(&self, tab: &Arc<Tab>) -> Result<()> {
        tab.evaluate("window.scrollTo(0, document.body.scrollHeight)", false)?;
        std::thread::sleep(Duration::from_secs(2));
        Ok(())
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
