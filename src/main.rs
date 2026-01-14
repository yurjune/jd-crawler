use jd_crawler::{JobCategory, JobSubcategory, Result, WantedClient, save_to_csv};

fn main() -> Result<()> {
    let client = WantedClient::new(4, JobCategory::Development, JobSubcategory::Frontend);
    let jobs = client.start_crawl(1)?;

    let csv_path = "wanted-frontend-jobs.csv";
    save_to_csv(&jobs, csv_path)?;
    println!("CSV 파일 저장 완료: {}", csv_path);

    Ok(())
}
