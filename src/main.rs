use jd_crawler::{Result, WantedClient};

fn main() -> Result<()> {
    println!("=== Wanted 프론트엔드 채용공고 크롤러 ===\n");

    let client = WantedClient::new();

    println!("프론트엔드 0~5년차 공고 조회 중...");
    let jobs = client.fetch_frontend_jobs(0, 5)?;

    println!("\n✅ 총 {}개의 채용공고를 찾았습니다.\n", jobs.len());

    for (i, job) in jobs.iter().enumerate() {
        println!("{}. {}", i + 1, job.title);
        println!("회사: {}", job.company);
        println!("경력: {}", job.experience_years);
        println!("URL: {}", job.url);
        println!();
    }

    Ok(())
}
