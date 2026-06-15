use anyhow::Result;

use esp_idf_svc::http::server::{
    EspHttpServer,
};
use esp_idf_svc::http::Method;


const STYLE_CSS: &str = include_str!("../../styles/style.css");
const HOME: &str = include_str!("../../templates/home.html");

pub fn load_urls(
    http: &mut EspHttpServer,
) -> Result<()> {
    http.fn_handler("/style.css", Method::Get, move |request| -> Result<()> {
        let mut response = request.into_response(
            200,
            Some("OK"),
            &[("Content-Type", "text/css")],
        )?;

        response.write(STYLE_CSS.as_bytes())?;
        Ok(())
    })?;
    
    http.fn_handler("/", Method::Get, move |request| -> Result<()> {
        let mut response = request.into_ok_response()?;
        response.write(HOME.as_bytes())?;
        
        Ok(())
    })?;
    
    Ok(())
}
