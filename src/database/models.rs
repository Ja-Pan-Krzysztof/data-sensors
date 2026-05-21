use sqlx::FromRow;
use chrono::{DateTime, Utc};


#[derive(Debug, FromRow, Clone)]
pub struct Network {
    pub id: i32,
    pub ssid: String,
    pub password: String,
    pub is_active: bool,
}

#[derive(Debug, FromRow, Clone)]
pub struct Sensor {
    pub id: i32,
    pub name: String,
    pub sensor_type: String,
    pub value: f32,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, FromRow, Clone)]
pub struct Alarm {
    pub id: i32,
    pub sensor_id: i32,
    pub is_triggered: bool,
    pub created: DateTime<Utc>,
}
