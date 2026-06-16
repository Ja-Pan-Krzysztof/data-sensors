use anyhow::Result;

use esp_idf_hal::modem::Modem;
use esp_idf_hal::delay::FreeRtos;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::{
        server::{Configuration as HttpServerConfig, EspHttpServer}
    },
    nvs::EspDefaultNvsPartition,
    wifi::{
        BlockingWifi,
        ClientConfiguration,
        Configuration,
        EspWifi,
    },
};

use std::sync::{Arc, Mutex};
use std::default::Default;

use crate::config::{LiveMeasurements, Screen};

pub type Wifi = BlockingWifi<EspWifi<'static>>;
pub type Server = EspHttpServer<'static>;


pub fn run_server(
    modem: Modem,
    ssid: &str,
    pass: &str,
    shared_oled: Arc<Mutex<Screen>>,
    shared_data: Arc<Mutex<LiveMeasurements>>,
) -> Result<(Wifi, Server)> {
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        password: pass.try_into().unwrap(),
        ..Default::default()
    }))?;

    wifi.start()?;
    let mut attempt = 1;

    loop {
        if let Ok(mut oled) = shared_oled.lock() {
            let mess = format!("[{}] Connecting to {}", attempt, ssid);

            if let Err(e) = oled.clear() {
                println!("[OLED ERROR] Clear failed: {:?}", e);
            }

            if let Err(e) = oled.show_text(&mess, 5, 10) {
                println!("[OLED ERROR] Show text failed: {:?}", e);
            }

            if let Err(e) = oled.refresh() {
                println!("[OLED ERROR] Refresh failed: {:?}", e);
            }
        }

        match wifi.connect() {
            Ok(_) => {
                match wifi.wait_netif_up() {
                    Ok(_) => break,
                    Err(wifi_err) => println!("Cannot get IP address from router DHCP: {:?}", wifi_err),
                }
            },
            Err(esp_err) => println!("Cannot connect to your WiFi: {:?}", esp_err),
        }

        let _ = wifi.disconnect();
        attempt += 1;
        FreeRtos::delay_ms(1500);
    }

    let ip = wifi.wifi().sta_netif().get_ip_info()?.ip;

    if let Ok(mut oled) = shared_oled.lock() {
        let mess = format!("Connected with {}", ip);

        if let Err(e) = oled.clear() {
            println!("[OLED ERROR] Clear failed: {:?}", e);
        }

        if let Err(e) = oled.show_text(&mess, 10, 20) {
            println!("[OLED ERROR] Show text failed: {:?}", e);
        }

        if let Err(e) = oled.refresh() {
            println!("[OLED ERROR] Refresh failed: {:?}", e);
        }
    }

    let mut server = EspHttpServer::new(&HttpServerConfig::default())?;

    crate::website::urls::load_urls(&mut server, shared_data.clone())?;

    Ok((wifi, server))
}
