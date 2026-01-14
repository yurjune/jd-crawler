use jd_crawler::{Result, WantedClient};

fn main() -> Result<()> {
    println!("=== Wanted 프론트엔드 채용공고 크롤러 ===\n");

    let client = WantedClient::new();

    println!("프론트엔드 0~5년차 공고 조회 중...");
    let html = client.fetch_frontend_jobs(0, 5)?;

    println!("\n✅ HTML 수신 완료 ({} bytes)", html.len());
    println!("\n=== HTML 샘플 (처음 2000자) ===");
    println!("{}", &html[..html.len().min(2000)]);

    Ok(())
}
