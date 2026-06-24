use anyhow::Result;

use std::sync::{Arc, Mutex};

use esp_idf_svc::http::server::{
    EspHttpServer,
};
use esp_idf_svc::http::Method;

use crate::config::LiveMeasurements;
use crate::database::repository::SensorRepository;
use crate::network::SettingsUpdate;

const HOME: &str = include_str!("../../templates/home.html");
const SETTINGS: &str = include_str!("../../templates/settings.html");
const STYLE_CSS: &str = include_str!("../../styles/style.css");
const SCRIPT: &str = include_str!("../../scripts/script.js");

pub fn load_urls(
    http: &mut EspHttpServer,
    shared_data: Arc<Mutex<LiveMeasurements>>,
    db_repository: Arc<SensorRepository>
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

    http.fn_handler("/settings", Method::Get, move |request| -> Result<()> {
        let mut response = request.into_ok_response()?;
        response.write(SETTINGS.as_bytes())?;

        Ok(())
    })?;

    let db_repo_get = db_repository.clone();
    http.fn_handler("/api/sensors", Method::Get, move |request| -> Result<()> {
        let sensors = db_repo_get.get_all_sensors().unwrap_or_default();

        let mut json_string = String::from("[");
        for (i, s) in sensors.iter().enumerate() {
            let comma = if i == sensors.len() - 1 { "" } else { "," };
            // Formatujemy każdego sensora do obiektu JSON
            let sensor_json = format!(
                "{{\"id\":{},\"name\":\"{}\",\"value\":{:.2},\"min_threshold\":{:.2},\"max_threshold\":{:.2}}}",
                s.id, s.name, s.value, s.min_threshold, s.max_threshold
            );
            json_string.push_str(&sensor_json);
            json_string.push_str(comma);
        }
        json_string.push_str("]");

        let mut response = request.into_response(
            200,
            Some("OK"),
            &[
                ("Content-Type", "application/json"),
                ("Cache-Control", "no-cache"),
            ],
        )?;

        response.write(json_string.as_bytes())?;
        // response.flush()?;

        Ok(())
    })?;

    let db_repo_post = db_repository.clone();
    http.fn_handler("/api/settings", Method::Post, move |mut request| -> Result<()> {
        let mut buf = vec![0; 512];
        let bytes_read = request.read(&mut buf)?;
        let body = std::str::from_utf8(&buf[..bytes_read]).unwrap_or("");

        let parts: Vec<&str> = body.split(',').collect();

        if parts.len() == 3 {
            if let (Ok(sensor_id), Ok(min_val), Ok(max_val)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<f32>(),
                parts[2].parse::<f32>(),
            ) {
                if let Ok(sensors) = db_repo_post.get_all_sensors() {
                    if let Some(sensor) = sensors.iter().find(|s| s.id == sensor_id) {
                        let _ = db_repo_post.update_sensor_config(
                          sensor_id,
                          min_val,
                          max_val,
                          sensor.sensor_type.clone()
                        );
                    }
                }

                let mut response = request.into_response(200, Some("OK"), &[("Content-Type", "application/json")])?;
                response.write(b"{\"status\": \"Save Successfully\"}")?;

                 return Ok(());
            }
        }

        let mut response = request.into_response(400, Some("Bad request"), &[("Content-Type", "application/json")])?;
        response.write(b"{\"error\": \"Incorrect data format\"}")?;

        Ok(())
    })?;
    
    Ok(())
}
