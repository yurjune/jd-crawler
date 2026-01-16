use jd_crawler::{
    BlindEnricher, EnricherConfig, SaraminClient, SaraminCrawlConfig, SaraminJobCategory,
};
use jd_crawler::{CrawlPipeline, Result};
use jd_crawler::{WantedClient, WantedCrawlConfig, WantedJobCategory, WantedJobSubcategory};

fn main() -> Result<()> {
    CrawlPipeline::new()
        .crawl(WantedClient::new(WantedCrawlConfig {
            category: WantedJobCategory::Development,
            subcategory: WantedJobSubcategory::Frontend,
            total_pages: 50,
            min_years: 0,
            max_years: 5,
            full_crawl: true,
            thread_count: 8,
        }))?
        .save_and_then("wanted.csv")
        .enrich(BlindEnricher::new(EnricherConfig { thread_count: 1 }))
        .save("wanted.csv");

    CrawlPipeline::new()
        .crawl(SaraminClient::new(SaraminCrawlConfig {
            category: SaraminJobCategory::Frontend,
            total_pages: 50,
            thread_count: 8,
        }))?
        .save_and_then("saramin.csv")
        .enrich(BlindEnricher::new(EnricherConfig { thread_count: 1 }))
        .save("saramin.csv");

    Ok(())
}
