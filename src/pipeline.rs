use crate::crawler::DetailCrawler;
use crate::enricher::JobEnricher;
use crate::writer::save_to_csv;
use crate::{Job, Result};

pub struct DetailFetcherConfig {
    pub thread_count: usize,
    pub includes: Vec<&'static str>,
}

pub struct CrawlPipeline;

#[must_use = "pipeline must end with .save() to execute"]
pub struct PipelineWithJobs<C> {
    jobs: Vec<Job>,
    client: C,
}

impl CrawlPipeline {
    pub fn new() -> Self {
        Self
    }

    pub fn crawl<C>(self, client: C) -> Result<PipelineWithJobs<C>>
    where
        C: Crawler,
    {
        let jobs = client.start_crawl()?;
        Ok(PipelineWithJobs { jobs, client })
    }
}

impl Default for CrawlPipeline {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Crawler {
    fn start_crawl(&self) -> Result<Vec<Job>>;
}

impl<C> PipelineWithJobs<C> {
    pub fn enrich(mut self, enricher: impl JobEnricher + 'static) -> Self {
        if let Ok(enriched) = enricher.start_enrich(&self.jobs) {
            self.jobs = enriched
        }
        self
    }

    #[must_use = "save_and_then() returns Self to allow chaining"]
    pub fn save_and_then(self, path: impl Into<String>) -> Self {
        let path = path.into();
        match save_to_csv(&self.jobs, &path) {
            Ok(_) => println!("✅ csv 저장 완료: {}", path),
            Err(e) => eprintln!("❌ csv 저장 실패 ({}): {}", path, e),
        }
        self
    }

    pub fn save(self, path: impl Into<String>) {
        let path = path.into();
        match save_to_csv(&self.jobs, &path) {
            Ok(_) => println!("✅ csv 저장 완료: {}", path),
            Err(e) => eprintln!("❌ csv 저장 실패 ({}): {}", path, e),
        }
    }
}

impl<C> PipelineWithJobs<C>
where
    C: DetailCrawler,
{
    pub fn fetch_details(mut self, config: DetailFetcherConfig) -> Self {
        println!("상세 정보 수집 시작..");
        match self.client.fetch_details(self.jobs.clone(), config) {
            Ok(jobs_with_details) => {
                println!("✅ 상세 정보 수집 완료");
                self.jobs = jobs_with_details;
            }
            Err(e) => {
                eprintln!("❌ 상세 정보 수집 실패: {}", e);
            }
        }
        self
    }
}
