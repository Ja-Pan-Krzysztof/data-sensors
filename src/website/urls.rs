use anyhow::Result;

use std::sync::{Arc, Mutex};
use std::io::Write;

use esp_idf_svc::http::server::{
    EspHttpServer,
};
use esp_idf_svc::http::Method;

use crate::config::LiveMeasurements;


const STYLE_CSS: &str = include_str!("../../styles/style.css");
const HOME: &str = include_str!("../../templates/home.html");
const SCRIPT: &str = include_str!("../../scripts/script.js");

pub fn load_urls(
    http: &mut EspHttpServer,
    shared_data: Arc<Mutex<LiveMeasurements>>,
) -> Result<()> {
    http.fn_handler("/styles/style.css", Method::Get, move |request| -> Result<()> {
        let mut response = request.into_response(
            200,
            Some("OK"),
            &[("Content-Type", "text/css")],
        )?;

        response.write(STYLE_CSS.as_bytes())?;
        Ok(())
    })?;

    http.fn_handler("/scripts/script.js", Method::Get, move |request| -> Result<()> {
        let mut response = request.into_response(
            200,
            Some("OK"),
            &[("Content-Type", "application/javascript")],
        )?;

        response.write(SCRIPT.as_bytes())?;
        Ok(())
    })?;
    
    http.fn_handler("/", Method::Get, move |request| -> Result<()> {
        let mut response = request.into_ok_response()?;
        response.write(HOME.as_bytes())?;
        
        Ok(())
    })?;

    let data_clone = shared_data.clone();
    http.fn_handler("/api/data", Method::Get, move |request| -> Result<()> {
        let snapshot = if let Ok(data) = data_clone.lock() {
            *data
        } else {
            LiveMeasurements::default()
        };

        let json_string = format!(
            "{{\"temperature\":{},\"light_percent\":{},\"is_tilted\":{},\"shock_detected\":{},\"distance_cm\":{}}}",
            snapshot.temperature,
            snapshot.light_percent,
            snapshot.is_tilted,
            snapshot.shock_detected,
            snapshot.distance_cm
        );

        let mut response = request.into_response(
            200,
            Some("OK"),
            &[
                ("Content-Type", "application/json"),
                ("Cache-Control", "no-cache"),
            ],
        )?;

        response.write(json_string.as_bytes())?;
        response.flush()?;

        Ok(())
    })?;
    
    Ok(())
}
