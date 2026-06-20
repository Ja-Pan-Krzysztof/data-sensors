mod database;
mod config;
mod monitor;
mod oled;
mod exceptions;
mod website;
mod network;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::env;

use anyhow::Result;

use esp_idf_hal::prelude::Peripherals;

use crate::config::{LiveMeasurements, Screen, SensorsConfig};
use crate::database::models::{AppDatabase, Sensor};
use crate::database::repository::SensorRepository;
use crate::database::storage::init_storage;
use crate::monitor::SystemMonitor;

fn main() -> Result<()> {
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;

    let (oled, _res) = config::init_oled(
        peripherals.spi2,
        peripherals.pins.gpio1,
        peripherals.pins.gpio2,
        //peripherals.pins.gpio16,
        peripherals.pins.gpio15,
        peripherals.pins.gpio16,
        peripherals.pins.gpio17,
    )?;
    
    let db_path = "spiffs/db.json";
    if let Err(e) = init_storage() {
        println!("[DISK ERROR] -> Unable to init file: {:?}", e)
    }

    let hardware = SensorsConfig::init(
        peripherals.adc1,
        peripherals.pins.gpio4,
        peripherals.pins.gpio5,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
    )?;

    let shared_oled: Arc<Mutex<Screen>> = Arc::new(Mutex::new(oled));
    let shared_data: Arc<Mutex<LiveMeasurements>> = Arc::new(Mutex::new(LiveMeasurements::default()));
    let default_db = init_storage()?;
    let db_repository = SensorRepository::new(default_db);

    let (wifi, _server) = network::run_server(
        peripherals.modem,
        env!("SSID"),
        env!("PASSWORD"),
        shared_oled.clone(),
        shared_data.clone(),
    )?;

    let mut monitor = SystemMonitor::new(hardware, shared_data, db_repository);
    
    thread::Builder::new()
        .name("sensor_monitor_thread".to_string())
        .stack_size(32 * 1024)
        .spawn(move || {
            if let Err(e) = monitor.start_monitor() {
                println!("[MONITOR ERROR] -> start monitor error: {:?}", e);
            }
    })
        .expect("[MONITOR ERROR] -> Unable to create monitor thread");

    loop {
        if let Ok(mut display) = shared_oled.lock() {
            if let Err(e) = display.clear() { println!("[OLED ERROR] -> Clean failed: {:?}", e); }
            if let Err(e) = display.show_text("Working", 5, 20) { println!("[OLED ERROR] -> Draw failed: {:?}", e); }
            if let Err(e) = display.refresh() { println!("[OLED ERROR] -> Refresh failed: {:?}", e); }
        } else {
            println!("[ERROR] -> Mutex cannot lock screen")
        }

        thread::sleep(Duration::from_secs(1));
    }
}
