use crate::enricher::JobEnricher;
use crate::writer::save_to_csv;
use crate::{Job, Result};

pub struct CrawlPipeline;

#[must_use = "pipeline must end with .save() to execute"]
pub struct PipelineWithJobs {
    jobs: Vec<Job>,
}

impl CrawlPipeline {
    pub fn new() -> Self {
        Self
    }

    pub fn crawl<C>(self, client: C) -> Result<PipelineWithJobs>
    where
        C: Crawler,
    {
        let jobs = client.start_crawl()?;
        Ok(PipelineWithJobs { jobs })
    }
}

impl Default for CrawlPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineWithJobs {
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

pub trait Crawler {
    fn start_crawl(self) -> Result<Vec<Job>>;
}
