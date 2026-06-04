use esp_idf_hal::adc::oneshot::AdcDriver;
use esp_idf_hal::adc::ADC1;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;


pub struct HardwareConfig {
    pub adc_driver: AdcDriver<'static, ADC1>,
    pub temp_pin: Gpio4,
    pub light_pin: Gpio5,
    pub tilt_pin: PinDriver<'static, Gpio6, Input>,
    pub trig_pin: PinDriver<'static, Gpio7, Output>,
    pub echo_pin: PinDriver<'static, Gpio8, Input>,
}

impl HardwareConfig {
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

        Ok(Self {
            adc_driver,
            temp_pin,
            light_pin,
            tilt_pin,
            trig_pin,
            echo_pin,
        })
    }
}
