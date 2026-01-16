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
            total_pages: 1,
            min_years: 0,
            max_years: 5,
            full_crawl: false,
            thread_count: 8,
        }))?
        .enrich(BlindEnricher::new(EnricherConfig { thread_count: 8 }))
        .save("wanted-frontend-jobs.csv");

    CrawlPipeline::new()
        .crawl(SaraminClient::new(SaraminCrawlConfig {
            category: SaraminJobCategory::Frontend,
            total_pages: 16,
            thread_count: 8,
        }))?
        .enrich(BlindEnricher::new(EnricherConfig { thread_count: 1 }))
        .save("saramin-frontend-jobs.csv");

    Ok(())
}
