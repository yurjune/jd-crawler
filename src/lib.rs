pub mod crawler;
pub mod models;
pub mod saramin_client;
pub mod wanted_client;
pub mod writer;

pub use crawler::{JobCrawler, JobListInfiniteScrollCrawler, JobListPaginatedCrawler};
pub use models::{CrawlConfig, Job};
pub use saramin_client::{SaraminClient, SaraminJobCategory};
pub use wanted_client::{WantedClient, WantedJobCategory, WantedJobSubcategory};
pub use writer::save_to_csv;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
