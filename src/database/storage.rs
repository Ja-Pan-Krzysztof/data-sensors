use std::{fs, env};
use std::path::Path;
use chrono::Utc;
use crate::database::models::*;


pub fn init_storage(file_path: &str) -> anyhow::Result<()> {
    if Path::new(file_path).exists() {
        return Ok(())
    }
    
    let default_db = AppDatabase {
        network: Network {
            ssid: env::var("SSID").unwrap_or_else(|_| "0".to_string()),
            password: env::var("PASSWORD").unwrap_or_else(|_| "0".to_string()),
            is_active: false,
        },
        sensor: vec![
            Sensor { id: 1, name: "collision sensor".to_string(), sensor_type: "".to_string(), value: 0.0, min_threshold: 0.0, max_threshold: 0.0, updated: Utc::now() },
            Sensor { id: 2, name: "light sensor".to_string(), sensor_type: "".to_string(), value: 0.0, min_threshold: 0.0, max_threshold: 0.0, updated: Utc::now() },
            Sensor { id: 3, name: "tilt sensor".to_string(), sensor_type: "".to_string(), value: 0.0, min_threshold: 0.0, max_threshold: 0.0, updated: Utc::now() },
            Sensor { id: 4, name: "temp sensor".to_string(), sensor_type: "".to_string(), value: 0.0, min_threshold: 0.0, max_threshold: 0.0, updated: Utc::now() },
            Sensor { id: 5, name: "shock sensor".to_string(), sensor_type: "".to_string(), value: 0.0, min_threshold: 0.0, max_threshold: 0.0, updated: Utc::now() },
            Sensor { id: 6, name: "vibration sensor".to_string(), sensor_type: "".to_string(), value: 0.0, min_threshold: 0.0, max_threshold: 0.0, updated: Utc::now() },
        ],
        alarm: Vec::new(),
    };
    
    let serialized = serde_json::to_string_pretty(&default_db)?;
    fs::write(file_path, serialized)?;
    
    Ok(())
}
