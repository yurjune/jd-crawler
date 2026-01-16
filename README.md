# Job description crawler

Job description crawler for developer written in Rust

### 지원 사이트

- [원티드](https://www.wanted.co.kr) - 채용 공고
- [사람인](https://www.saramin.co.kr/) - 채용 공고
- [블라인드](https://www.teamblind.com/kr/) - 평점/리뷰

### CSV format


  | 컬럼             | 설명           |
  | ---------------- | -------------- |
  | title            | 채용 공고 제목 |
  | company          | 회사명         |
  | experience_years | 경력 요구사항  |
  | deadline         | 마감일         |
  | location         | 근무지         |
  | rating           | 평점           |
  | review_count     | 리뷰 개수      |
  | url              | 공고 링크      |


### 실행 방법 

```bash
cargo run 
```

### Examples

- 원티드 채용 공고 크롤링

```rust
fn main() -> Result<()> {
    CrawlPipeline::new()
        // 채용 공고 크롤링
        .crawl(WantedClient::new(WantedCrawlConfig {
            category: WantedJobCategory::Development,
            subcategory: WantedJobSubcategory::Frontend,
            total_pages: 12,
            min_years: 0,
            max_years: 5,
            full_crawl: false,
            thread_count: 8,
        }))?
        // 블라인드 평점/리뷰 기록
        .enrich(BlindEnricher::new(EnricherConfig { thread_count: 1 })) 
        .save("wanted-frontend-jobs.csv");

    Ok(())
}
```


- 사람인 채용 공고 크롤링

```rust
fn main() -> Result<()> {
    CrawlPipeline::new()
        // 채용 공고 크롤링
        .crawl(SaraminClient::new(SaraminCrawlConfig {
            category: SaraminJobCategory::Frontend,
            total_pages: 24,
            thread_count: 8,
        }))?
        // 블라인드 평점/리뷰 기록
        .enrich(BlindEnricher::new(EnricherConfig { thread_count: 1 }))
        .save("saramin-frontend-jobs.csv");

    Ok(())
}
```

