use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct Job {
    pub title: String,
    pub company: String,
    pub experience_years: String,
    pub deadline: String,
    pub location: String,
    pub url: String,
}
