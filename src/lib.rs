pub mod clients;
pub mod crawler;
pub mod enricher;
pub mod enrichers;
pub mod models;
pub mod pipeline;
pub mod utils;
pub mod writer;

pub use clients::{
    SaraminClient, SaraminCrawlConfig, SaraminJobCategory, WantedClient, WantedCrawlConfig,
    WantedJobCategory, WantedJobSubcategory,
};
pub use crawler::{
    DetailCrawlConfig, JobCrawler, JobListInfiniteScrollCrawler, JobListPaginatedCrawler,
};
pub use enricher::{EnricherConfig, JobEnricher};
pub use enrichers::BlindEnricher;
pub use models::Job;
pub use pipeline::CrawlPipeline;
pub use writer::save_to_csv;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
