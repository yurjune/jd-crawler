use crate::crawler::JobCrawler;
use crate::{Job, Result};
use headless_chrome::Tab;
use scraper::{Html, Selector};
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
        self.fetch_all_jobs(&url)
    }

    fn build_url(&self, min_years: u8, max_years: u8) -> String {
        format!(
            "{}/wdlist/{}/{}?country=kr&job_sort=job.recommend_order&years={}&years={}&locations=all",
            self.base_url, JOB_CATEGORY_DEVELOPMENT, JOB_SUBCATEGORY_FRONTEND, min_years, max_years
        )
    }
}

impl JobCrawler for WantedClient {
    fn wait_for_page_load(&self, tab: &Arc<Tab>) -> Result<()> {
        println!("페이지 로딩 대기 중...");
        tab.wait_for_element("body")?;
        tab.wait_for_element(r#"[data-cy="job-list"]"#)?;
        std::thread::sleep(Duration::from_secs(3));
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
