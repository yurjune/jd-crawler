pub mod crawler;
pub mod models;
pub mod wanted_client;
pub mod writer;

pub use crawler::{JobCrawler, JobDetailCrawler, JobListCrawler};
pub use models::{CrawlConfig, Job};
pub use wanted_client::{WantedClient, WantedJobCategory, WantedJobSubcategory};
pub use writer::save_to_csv;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
