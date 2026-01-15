use serde::Serialize;

#[derive(Debug, Clone)]
pub struct CrawlConfig {
    /// 크롤링할 페이지 수
    pub total_pages: usize,
    /// 병렬 처리에 사용할 스레드 개수
    pub num_threads: usize,
    /// 최소 경력 (년)
    pub min_years: u8,
    /// 최대 경력 (년)
    pub max_years: u8,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            total_pages: 1,
            num_threads: 4,
            min_years: 0,
            max_years: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub title: String,
    pub company: String,
    pub experience_years: String,
    pub url: String,
    pub deadline: Option<String>,
    pub location: Option<String>,
}

impl Job {
    pub fn new(title: String, company: String, experience_years: String, url: String) -> Self {
        Self {
            title,
            company,
            experience_years,
            url,
            deadline: None,
            location: None,
        }
    }
}
