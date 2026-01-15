use jd_crawler::{
    CrawlConfig, Result, SaraminClient, SaraminJobCategory, WantedClient, WantedJobCategory,
    WantedJobSubcategory, save_to_csv,
};

fn main() -> Result<()> {
    // Wanted
    let client = WantedClient::new(
        WantedJobCategory::Development,
        WantedJobSubcategory::Frontend,
    );
    let jobs = client.start_crawl(CrawlConfig {
        total_pages: 1,
        num_threads: 4,
        min_years: 0,
        max_years: 5,
    })?;
    let csv_path = "wanted-frontend-jobs.csv";
    save_to_csv(&jobs, csv_path)?;

    // Saramin
    let client = SaraminClient::new(SaraminJobCategory::Frontend);
    let jobs = client.start_crawl(CrawlConfig {
        total_pages: 12,
        num_threads: 4,
        min_years: 0,
        max_years: 5,
    })?;
    let csv_path = "saramin-frontend-jobs.csv";
    save_to_csv(&jobs, csv_path)?;
    Ok(())
}
