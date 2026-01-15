pub mod saramin;
pub mod wanted;

pub use saramin::{SaraminClient, SaraminCrawlConfig, SaraminJobCategory};
pub use wanted::{WantedClient, WantedCrawlConfig, WantedJobCategory, WantedJobSubcategory};
