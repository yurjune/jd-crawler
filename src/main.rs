use jd_crawler::{Result, WantedClient, save_to_csv};

fn main() -> Result<()> {
    println!("=== Wanted 프론트엔드 채용공고 크롤러 ===\n");

    let client = WantedClient::new(4);

    println!("프론트엔드 0~5년차 공고 조회 중...");
    let jobs = client.fetch_frontend_jobs(0, 5, 1)?;

    println!("\n✅ 총 {}개의 채용공고를 찾았습니다.\n", jobs.len());

    let csv_path = "jobs.csv";
    save_to_csv(&jobs, csv_path)?;
    println!("CSV 파일 저장 완료: {}", csv_path);

    Ok(())
}
