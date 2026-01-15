use jd_crawler::{
    CrawlConfig, Result, WantedClient, WantedJobCategory, WantedJobSubcategory, save_to_csv,
};

fn main() -> Result<()> {
    let client = WantedClient::new(
        WantedJobCategory::Development,
        WantedJobSubcategory::Frontend,
    );
    let config = CrawlConfig {
        total_pages: 1,
        num_threads: 4,
        min_years: 0,
        max_years: 5,
    };
    let jobs = client.start_crawl(config)?;

    let csv_path = "wanted-frontend-jobs.csv";
    save_to_csv(&jobs, csv_path)?;
    println!("CSV 파일 저장 완료: {}", csv_path);

    Ok(())
}
