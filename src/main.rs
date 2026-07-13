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
use esp_idf_hal::uart::{UartConfig, UartDriver, config::DataBits, config::StopBits};
use esp_idf_hal::prelude::FromValueType;

use crate::config::{LiveMeasurements, Screen, SensorsConfig};
use crate::database::repository::SensorRepository;
use crate::database::storage::init_storage;
use crate::monitor::SystemMonitor;
use crate::exceptions::SensorCode;


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
    let db_repository = Arc::new(SensorRepository::new(default_db));

    let (wifi, _server) = network::run_server(
        peripherals.modem,
        env!("SSID"),
        env!("PASSWORD"),
        shared_oled.clone(),
        shared_data.clone(),
        db_repository.clone(),
    )?;

    let uart_config = UartConfig::new()
        .baudrate(115_200_u32.Hz())
        .data_bits(DataBits::DataBits8)
        .stop_bits(StopBits::STOP1);

    let uart = UartDriver::new(
        peripherals.uart0,
        peripherals.pins.gpio43,
        peripherals.pins.gpio44,
        Option::<esp_idf_hal::gpio::AnyInputPin>::None,
        Option::<esp_idf_hal::gpio::AnyOutputPin>::None,
        &uart_config,
    )?;

    let mut monitor = SystemMonitor::new(
        hardware,
        shared_data.clone(),
        db_repository.clone(),
        uart,
    );

    thread::Builder::new()
        .name("sensor_monitor_thread".to_string())
        .stack_size(32 * 1024)
        .spawn(move || {
            if let Err(e) = monitor.start_monitor() {
                println!("[MONITOR ERROR] -> start monitor error: {:?}", e);
            }
    })
        .expect("[MONITOR ERROR] -> Unable to create monitor thread");

    let loop_delay = Duration::from_millis(200);

    loop {
        let (temp, light, tilt, shock, dist, err_temp, err_light, err_dist) = {
            if let Ok(data) = shared_data.lock() {
                (
                    data.temperature,
                    data.light_percent,
                    data.is_tilted,
                    data.shock_detected,
                    data.distance_cm,
                    data.err_temp,
                    data.err_light,
                    data.err_dist,
                )
            } else {
                (0.0, 0.0, false, false, 0.0, SensorCode::Ok, SensorCode::Ok, SensorCode::Ok)
            }
        };

        if let Ok(mut display) = shared_oled.lock() {
            if let Err(e) = display.clear() { println!("[OLED ERROR] -> Clean failed: {:?}", e); }
            if let Err(e) = display.show_text(&format!("{:?}", wifi.wifi().sta_netif().get_ip_info()?.ip), 5, 10) { println!("[OLED ERROR] -> Draw failed: {:?}", e); }

            let txt_temp = if err_temp == SensorCode::Ok {
                format!("Temp: {:.1}C", temp)
            } else {
                format!("S1: {:?}", err_temp)
            };
            let _ = display.show_text(&txt_temp, 5, 22);

            let txt_light = if err_light == SensorCode::Ok {
                format!("Light: {:.0}%", light)
            } else {
                format!("S2: {:?}", err_light)
            };
            let _ = display.show_text(&txt_light, 5, 34);

            let txt_dist = if err_dist == SensorCode::Ok {
                format!("Dist: {:.1}cm", dist)
            } else {
                format!("S5: {:?}", err_dist)
            };
            let _ = display.show_text(&txt_dist, 5, 46);

            let t_str = if tilt { "YES" } else { "NO" };
            let s_str = if shock { "YES" } else { "NO" };
            let txt_motion = format!("TILT:{} SHK:{}", t_str, s_str);
            let _ = display.show_text(&txt_motion, 5, 58);

            if let Err(e) = display.refresh() { println!("[OLED ERROR] -> Refresh failed: {:?}", e); }
        } else {
            println!("[ERROR] -> Mutex cannot lock screen")
        }

        thread::sleep(loop_delay);
    }
}
