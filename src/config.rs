use anyhow::Result;

use esp_idf_hal::adc::oneshot::AdcDriver;
use esp_idf_hal::adc::ADC1;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::{self, SpiDeviceDriver, SpiDriver, SPI2};

use display_interface_spi::SPIInterface;
use esp_idf_hal::units::Hertz;
use crate::oled::OledDisplay;


/// Declarate pins
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
    pub fn init() -> anyhow::Result<Self> {
        let peripherals = Peripherals::take()?;

        let adc_driver = AdcDriver::new(peripherals.adc1)?;
        let temp_pin = peripherals.pins.gpio4;
        let light_pin = peripherals.pins.gpio5;

        let mut tilt_pin = PinDriver::input(peripherals.pins.gpio6)?;
        tilt_pin.set_pull(Pull::Up)?;

        let mut trig_pin = PinDriver::output(peripherals.pins.gpio7)?;
        let echo_pin = PinDriver::input(peripherals.pins.gpio8)?;
        trig_pin.set_low()?;
        
        let mut vibration_pin = PinDriver::input(peripherals.pins.gpio9)?;
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

pub type SpiInterfaceType = SPIInterface<
    SpiDeviceDriver<'static, SpiDriver<'static>>,
    PinDriver<'static, Gpio14, Output>
>;
pub type Screen = OledDisplay<SpiInterfaceType>;
pub type ResetPin = PinDriver<'static, Gpio15, Output>;

pub fn init_oled(
    spi: SPI2,
    sclk: Gpio10,
    sda: Gpio11,
    sdi: Gpio12,
    cs: Gpio13,
    dc: Gpio14,
    res: Gpio15,
) -> Result<(Screen, ResetPin)> {
    let driver_config = spi::config::DriverConfig::new();
    let device_config = spi::config::Config::new().baudrate(Hertz(4_000_000));
    let spi_device = SpiDeviceDriver::new_single(
        spi, sclk, sda, Some(sdi), Some(cs),
        &driver_config,
        &device_config,
    )?;

    let mut res = PinDriver::output(res)?;
    let dc = PinDriver::output(dc)?;

    let interface = SPIInterface::new(spi_device, dc);
    let screen = OledDisplay::new(interface, &mut res)?;

    Ok((screen, res))
}
