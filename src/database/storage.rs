use anyhow::Result;

use std::env;
use chrono::Utc;

use crate::database::models::*;


pub fn init_storage() -> Result<AppDatabase> {
    let default_db = AppDatabase {
        network: Network {
            ssid: env::var("SSID").unwrap_or_else(|_| "0".to_string()),
            password: env::var("PASSWORD").unwrap_or_else(|_| "0".to_string()),
            is_active: false,
        },
        sensor: vec![
            Sensor { id: 1, name: "Temp sensor".to_string(), sensor_type: "adc".to_string(), value: 0.0, min_threshold: 15.0, max_threshold: 40.0, updated: Utc::now() },
            Sensor { id: 2, name: "Light sensor".to_string(), sensor_type: "adc".to_string(), value: 0.0, min_threshold: 20.0, max_threshold: 60.0, updated: Utc::now() },
            Sensor { id: 3, name: "Tilt sensor".to_string(), sensor_type: "digital".to_string(), value: 0.0, min_threshold: 0.0, max_threshold: 1.0, updated: Utc::now() },
            Sensor { id: 4, name: "Shock sensor".to_string(), sensor_type: "digital".to_string(), value: 0.0, min_threshold: 0.0, max_threshold: 1.0, updated: Utc::now() },
            Sensor { id: 5, name: "Collision sensor".to_string(), sensor_type: "digital".to_string(), value: 400.0, min_threshold: 30.0, max_threshold: 200.0, updated: Utc::now() },
        ],
        alarm: Vec::new(),
    };
    
    Ok(default_db)
}
