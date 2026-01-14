pub mod client;
pub mod crawler;
pub mod models;
pub mod writer;

pub use client::WantedClient;
pub use crawler::JobCrawler;
pub use models::Job;
pub use writer::save_to_csv;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
