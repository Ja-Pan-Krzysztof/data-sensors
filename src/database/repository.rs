use std::fs;
use crate::database::models::{Alarm, AppDatabase, Sensor};
use anyhow;
use anyhow::Context;

pub struct SensorRepository {
    file_path: String,
}

impl SensorRepository {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }

    /// Load entire database to RAM memory
    pub fn load_db(&self) -> anyhow::Result<AppDatabase> {
        let content = fs::read_to_string(&self.file_path)
            .context("Cannot read file from flash memory")?;

        let db: AppDatabase = serde_json::from_str(&content)
            .context("Parsing error")?;

        Ok(db)
    }

    /// Write database to flash after updated them
    fn save_db(&self, db: &AppDatabase) -> anyhow::Result<()> {
        let serialized = serde_json::to_string_pretty(db)?;
        fs::write(&self.file_path, serialized)
            .context("Flash memory error")?;

        Ok(())
    }

    
}
