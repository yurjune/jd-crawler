use jd_crawler::{Result, save_to_csv};
use jd_crawler::{SaraminClient, SaraminCrawlConfig, SaraminJobCategory};
use jd_crawler::{WantedClient, WantedCrawlConfig, WantedJobCategory, WantedJobSubcategory};

fn main() -> Result<()> {
    // Wanted
    let client = WantedClient::new(
        WantedJobCategory::Development,
        WantedJobSubcategory::Frontend,
    );
    let jobs = client.start_crawl(WantedCrawlConfig {
        total_pages: 1,
        num_threads: 4,
        min_years: 0,
        max_years: 5,
        full_crawl: false,
    })?;
    let csv_path = "wanted-frontend-jobs.csv";
    save_to_csv(&jobs, csv_path)?;

    // Saramin
    let client = SaraminClient::new(SaraminJobCategory::Frontend);
    let jobs = client.start_crawl(SaraminCrawlConfig {
        total_pages: 8,
        num_threads: 8,
    })?;
    let csv_path = "saramin-frontend-jobs.csv";
    save_to_csv(&jobs, csv_path)?;

    Ok(())
}
