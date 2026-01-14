pub mod crawler;
pub mod models;
pub mod user_agent;
pub mod wanted_client;
pub mod writer;

pub use crawler::JobCrawler;
pub use models::Job;
pub use wanted_client::WantedClient;
pub use writer::save_to_csv;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
