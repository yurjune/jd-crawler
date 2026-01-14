use crate::{Job, Result};
use std::fs::File;

pub fn save_to_csv(jobs: &[Job], file_path: &str) -> Result<()> {
    let file = File::create(file_path)?;
    let mut writer = csv::Writer::from_writer(file);

    for job in jobs {
        writer.serialize(job)?;
    }

    writer.flush()?;
    Ok(())
}
