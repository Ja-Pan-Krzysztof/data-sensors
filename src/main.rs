mod database;
mod config;
mod monitor;
mod oled;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use anyhow;

use esp_idf_hal::prelude::Peripherals;

use crate::config::{Screen, SensorsConfig};
use crate::database::storage::init_storage;
use crate::monitor::SystemMonitor;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_hal::sys::link_patches();
    let peripherals = Peripherals::take()?;

    let (oled, _res) = config::init_oled(
        peripherals.spi2,
        peripherals.pins.gpio10,
        peripherals.pins.gpio11,
        peripherals.pins.gpio12,
        peripherals.pins.gpio13,
        peripherals.pins.gpio14,
        peripherals.pins.gpio15,
    )?;
    
    // let db_path = "db.json";
    // init_storage(db_path)?;

    let hardware = SensorsConfig::init()?;
    let mut monitor = SystemMonitor::new(hardware);
    
    monitor.start_monitor()?;

    let shared_oled: Arc<Mutex<Screen>> = Arc::new(Mutex::new(oled));

    loop {
        if let Ok(mut display) = shared_oled.lock() {
            display.clear()?;
            display.show_text("hello", 5, 20)?;
            display.refresh()?;
        } else {
            println!("[ERROR] -> Mutex cannot lock screen")
        }

        thread::sleep(Duration::from_secs(1));
    }
}
