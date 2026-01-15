pub mod clients;
pub mod crawler;
pub mod models;
pub mod writer;

pub use clients::{
    SaraminClient, SaraminCrawlConfig, SaraminJobCategory, WantedClient, WantedCrawlConfig,
    WantedJobCategory, WantedJobSubcategory,
};
pub use crawler::{JobCrawler, JobListInfiniteScrollCrawler, JobListPaginatedCrawler};
pub use models::Job;
pub use writer::save_to_csv;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
