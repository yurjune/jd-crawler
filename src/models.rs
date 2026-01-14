#[derive(Debug, Clone)]
pub struct Job {
    pub title: String,
    pub company: String,
    pub experience_years: String,
    pub url: String,
}

impl Job {
    pub fn new(title: String, company: String, experience_years: String, url: String) -> Self {
        Self {
            title,
            company,
            experience_years,
            url,
        }
    }
}
