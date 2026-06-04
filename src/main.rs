mod database;
mod config;
mod monitor;

use std::sync::{Arc, Mutex};
use anyhow;
use crate::config::HardwareConfig;
use crate::database::storage::init_storage;
use crate::monitor::SystemMonitor;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::log::EspLogger::initialize_default();
    
    // let db_path = "db.json";
    // init_storage(db_path)?;

    let hardware = HardwareConfig::init()?;
    let mut monitor = SystemMonitor::new(hardware);
    
    monitor.start_monitor()?;
    
    Ok(())
}
