use std::fs;
use anyhow;
use anyhow::Context;
use chrono::Utc;

use crate::database::models::{Alarm, AppDatabase, Sensor, Network};


pub struct SensorRepository {
    file_path: String,
}

impl SensorRepository {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }

    /// Load entire database to RAM memory
    fn load_db(&self) -> anyhow::Result<AppDatabase> {
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

    //

    /// Get list of all sensors
    pub fn get_all_sensors(&self) -> anyhow::Result<Vec<Sensor>> {
        let db = self.load_db()?;

        Ok(db.sensor)
    }

    /// Save new actual value to database
    pub fn update_sensor_value(&self, sensor_id: i32, new_value: f32) -> anyhow::Result<()> {
        let mut db = self.load_db()?;

        if let Some(i) = db.sensor.iter_mut().find(|i| i.id == sensor_id) {
            i.value = new_value;
            i.updated = Utc::now();
            self.save_db(&db)?;
        }

        Ok(())
    }

    /// Save new min and max value to sensor
    pub fn update_sensor_config(&self, sensor_id: i32, min: f32, max: f32, sensor_type: String) -> anyhow::Result<()> {
        let mut db = self.load_db()?;

        if let Some(i) = db.sensor.iter_mut().find(|i| i.id == sensor_id) {
            i.min_threshold = min;
            i.max_threshold = max;
            i.sensor_type = sensor_type;
            i.updated = Utc::now();
            self.save_db(&db)?;
        }

        Ok(())
    }

    //

    pub fn trigger_alarm(&self, sensor_id: i32, is_triggered: bool) -> anyhow::Result<()> {
        let mut db = self.load_db()?;

        // generate next ID for alarm
        let next_id = db.alarm.iter_mut().map(|a| a.id).max().unwrap_or(0) + 1;

        let new_alarm = Alarm {
            id: next_id,
            sensor_id,
            is_triggered,
            created: Utc::now(),
        };

        db.alarm.push(new_alarm);
        self.save_db(&db)?;

        Ok(())
    }

    /// Get list of alarms
    pub fn get_alarm_history(&self) -> anyhow::Result<Vec<Alarm>> {
        let db = self.load_db()?;

        Ok(db.alarm)
    }

    //

    /// Get network config
    pub fn get_network_config(&self) -> anyhow::Result<Network> {
        let db = self.load_db()?;

        Ok(db.network)
    }

    /// Update network config
    pub fn update_network_config(&self, new_network: Network) -> anyhow::Result<()> {
        let mut db = self.load_db()?;
        db.network = new_network;
        self.save_db(&db)?;
        Ok(())
    }
}
