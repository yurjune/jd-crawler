use jd_crawler::{
    BlindEnricher, DetailFetcherConfig, EnricherConfig, SaraminClient, SaraminCrawlConfig,
    SaraminJobCategory,
};
use jd_crawler::{CrawlPipeline, Result};
use jd_crawler::{WantedClient, WantedCrawlConfig, WantedJobCategory, WantedJobSubcategory};

fn main() -> Result<()> {
    CrawlPipeline::new()
        .crawl(WantedClient::new(WantedCrawlConfig {
            category: WantedJobCategory::Development,
            subcategory: WantedJobSubcategory::Frontend,
            total_pages: 2,
            min_years: 0,
            max_years: 5,
            thread_count: 8,
            exclude_keywords: vec!["IOS", "안드로이드", "5년 이상"],
        }))?
        .fetch_details(DetailFetcherConfig { thread_count: 8 })
        .save_and_then("wanted.csv")
        .enrich(BlindEnricher::new(EnricherConfig { thread_count: 1 }))
        .save("wanted.csv");

    CrawlPipeline::new()
        .crawl(SaraminClient::new(SaraminCrawlConfig {
            category: SaraminJobCategory::Frontend,
            total_pages: 8,
            thread_count: 8,
            exclude_keywords: vec!["IOS", "안드로이드", "5년 이상"],
        }))?
        .save_and_then("saramin.csv")
        .enrich(BlindEnricher::new(EnricherConfig { thread_count: 1 }))
        .save("saramin.csv");

    Ok(())
}
