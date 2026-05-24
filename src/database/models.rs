use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub ssid: String,
    pub password: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
    pub id: i32,
    pub name: String,
    pub sensor_type: String,
    pub value: f32,
    pub min_threshold: f32,
    pub max_threshold: f32,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub id: i32,
    pub sensor_id: i32,
    pub is_triggered: bool,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDatabase {
    pub network: Network,
    pub sensor: Vec<Sensor>,
    pub alarm: Vec<Alarm>,
}
