use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub title: String,
    pub company: String,
    pub experience_years: String,
    pub deadline: String,
    pub location: String,
    pub url: String,
}

impl Job {
    pub fn new(title: String, company: String, experience_years: String, url: String) -> Self {
        Self {
            title,
            company,
            experience_years,
            url,
            deadline: String::new(),
            location: String::new(),
        }
    }
}
