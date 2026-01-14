use serde::Serialize;

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

    pub fn with_details(mut self, deadline: Option<String>, location: Option<String>) -> Self {
        self.deadline = deadline;
        self.location = location;
        self
    }
}
