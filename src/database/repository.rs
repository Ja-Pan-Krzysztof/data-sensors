use std::sync::Mutex;

use anyhow::{anyhow, Result};
use chrono::Utc;

use crate::database::models::{Alarm, AppDatabase, Sensor};


pub struct SensorRepository {
    ram_db: Mutex<AppDatabase>,
}

impl SensorRepository {
    pub fn new(default_db: AppDatabase) -> Self {
        Self { ram_db: Mutex::new(default_db) }
    }

    /// Load entire database to RAM memory
    fn load_db(&self) -> Result<AppDatabase> {
        let db = self.ram_db.lock()
            .map_err(|_| anyhow!("[DB ERROR] -> Lock db ram error"))?;

        Ok(db.clone())
    }

    /// Write database to flash after updated them
    fn save_db(&self, db: &AppDatabase) -> Result<()> {
        let mut ram = self.ram_db.lock()
            .map_err(|_| anyhow!("[DB ERROR] -> Lock db ram error"))?;

        *ram = db.clone();
        Ok(())
    }

    /// Get list of all sensors
    pub fn get_all_sensors(&self) -> Result<Vec<Sensor>> {
        let db = self.load_db()?;

        Ok(db.sensor)
    }

    /// Save new min and max value to sensor
    pub fn update_sensor_config(&self, sensor_id: i32, min: f32, max: f32, sensor_type: String) -> Result<()> {
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

    pub fn trigger_alarm(&self, sensor_id: i32, is_triggered: bool) -> Result<()> {
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

    /*
    /// Save new actual value to database
    pub fn update_sensor_value(&self, sensor_id: i32, new_value: f32) -> Result<()> {
        let mut db = self.load_db()?;

        if let Some(i) = db.sensor.iter_mut().find(|i| i.id == sensor_id) {
            i.value = new_value;
            i.updated = Utc::now();
            self.save_db(&db)?;
        }

        Ok(())
    }

    /// Get list of alarms
    pub fn get_alarm_history(&self) -> Result<Vec<Alarm>> {
        let db = self.load_db()?;

        Ok(db.alarm)
    }

    /// Get network config
    pub fn get_network_config(&self) -> Result<Network> {
        let db = self.load_db()?;

        Ok(db.network)
    }

    /// Update network config
    pub fn update_network_config(&self, new_network: Network) -> Result<()> {
        let mut db = self.load_db()?;
        db.network = new_network;
        self.save_db(&db)?;
        Ok(())
    }
     */

    pub fn update_all_sensors_batch(
        &self,
        temp: f32,
        light: f32,
        tilt: f32,
        vibrations: f32,
        distance: f32,
    ) -> Result<()> {
        let mut db = self.load_db()?;
        let now = Utc::now();

        for sensor in db.sensor.iter_mut() {
            match sensor.id {
                1 => sensor.value = temp,
                2 => sensor.value = light,
                3 => sensor.value = tilt,
                4 => sensor.value = vibrations,
                5 => sensor.value = distance,
                _ => {},
            }

            sensor.updated = now;
        }

        self.save_db(&db)?;
        Ok(())
    }
}
