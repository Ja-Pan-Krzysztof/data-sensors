use anyhow::Result;

use esp_idf_hal::adc::oneshot::AdcDriver;
use esp_idf_hal::adc::ADC1;
use esp_idf_hal::gpio::*;
use esp_idf_hal::spi::{self, SpiDeviceDriver, SpiDriver, SPI2, Dma};

use display_interface_spi::SPIInterface;
use esp_idf_hal::units::Hertz;

use crate::oled::OledDisplay;
use crate::exceptions::OledErorr;


/// Assigning pins to specyfic sensors
pub struct SensorsConfig {
    pub adc_driver: AdcDriver<'static, ADC1>,
    pub temp_pin: Gpio4,
    pub light_pin: Gpio5,
    pub tilt_pin: PinDriver<'static, Gpio6, Input>,
    pub trig_pin: PinDriver<'static, Gpio7, Output>,
    pub echo_pin: PinDriver<'static, Gpio8, Input>,
    pub vibration_pin: PinDriver<'static, Gpio9, Input>,
}

/// Configuration pins
impl SensorsConfig {
    pub fn init(
        adc1: ADC1,
        temp_pin: Gpio4,
        light_pin: Gpio5,
        tilt_pin: Gpio6,
        trig_pin: Gpio7,
        echo_pin: Gpio8,
        vibration_pin: Gpio9,
    ) -> Result<Self> {
        let adc_driver = AdcDriver::new(adc1)?;
        let temp_pin = temp_pin;
        let light_pin = light_pin;

        let mut tilt_pin = PinDriver::input(tilt_pin)?;
        tilt_pin.set_pull(Pull::Up)?;

        let mut trig_pin = PinDriver::output(trig_pin)?;
        let echo_pin = PinDriver::input(echo_pin)?;
        trig_pin.set_low()?;
        
        let mut vibration_pin = PinDriver::input(vibration_pin)?;
        vibration_pin.set_pull(Pull::Up)?;

        Ok(Self {
            adc_driver,
            temp_pin,
            light_pin,
            tilt_pin,
            trig_pin,
            echo_pin,
            vibration_pin,
        })
    }
}

/// Screen config
pub type SpiInterfaceType = SPIInterface<
    SpiDeviceDriver<'static, SpiDriver<'static>>,
    PinDriver<'static, Gpio16, Output>
>;
pub type Screen = OledDisplay<SpiInterfaceType>;
pub type ResetPin = PinDriver<'static, Gpio17, Output>;

/// Create instance of screen
pub fn init_oled(
    spi: SPI2,
    sclk: Gpio1,
    sda: Gpio2,
    // sdi: Gpio15,
    cs: Gpio15,
    dc: Gpio16,
    res: Gpio17,
) -> Result<(Screen, ResetPin)> {
    let driver_config = spi::config::DriverConfig::new()
        .dma(Dma::Auto(4096));
    let device_config = spi::config::Config::new().baudrate(Hertz(4_000_000));
    let spi_device = SpiDeviceDriver::new_single(
        spi, sclk, sda, None::<AnyIOPin>, Some(cs),
        &driver_config,
        &device_config,
    )?;

    let mut res = PinDriver::output(res)?;
    let dc = PinDriver::output(dc)?;

    let interface = SPIInterface::new(spi_device, dc);
    let screen = OledDisplay::new(interface, &mut res)
        .map_err(|e| anyhow::anyhow!("[OLED CONFING ERROR] -> Init error: {:?}", e))?;

    Ok((screen, res))
}
