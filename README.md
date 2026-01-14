# Job description crawler

Job description crawler for developer written in Rust

### 지원 사이트

- [원티드](https://www.wanted.co.kr)

### 실행 방법 

```bash
cargo run 
```

### Examples

- 원티드 프론트엔드 채용 공고 크롤링

```rust
fn main() -> Result<()> {
    let client = WantedClient::new(JobCategory::Development, JobSubcategory::Frontend);
    let config = CrawlConfig {
        total_pages: 20,  // 크롤링할 페이지 수
        num_threads: 4,  // 가동 스레드 수
        min_years: 0,  // 최소 경력
        max_years: 5,  // 최대 경력
    };
    let jobs = client.start_crawl(config)?;

    let csv_path = "frontend.csv";
    save_to_csv(&jobs, csv_path)?;

    Ok(())
}
```


