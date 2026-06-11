use std::thread;
use std::time::{Duration, Instant};

use esp_idf_hal::adc::oneshot::AdcChannelDriver;
use esp_idf_hal::adc::oneshot::config::{AdcChannelConfig, Calibration};
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::delay::Ets;

use crate::config::SensorsConfig;
use crate::exceptions::{SensorResult, SensorCode};


pub struct SystemMonitor {
    // shared_state: Arc<Mutex<AppDatabase>>,
    hardware: SensorsConfig,
}

impl<T> SensorResult<T> {
    fn new(value: T, code: SensorCode) -> Self {
        Self { value, code }
    }
}

impl SystemMonitor {
    pub fn new(hardware: SensorsConfig) -> Self {
        Self { hardware }
        // Self { shared_state, hardware }
    }

    /// Temperature sensos
    /// Converts the raw ADC value from thermistor into degrees Celsius.
    ///
    /// # Arguments
    /// * `raw_value` - Raw reading from ADC1 converter (1 - 4095)
    ///
    /// # Returns
    /// * (`value` and `status code`)
    fn raw_to_celsius(&self, raw_value: u16) -> SensorResult<f32> {
        if raw_value >= 4095 {
            return SensorResult::new(raw_value as f32, SensorCode::VoltageToHigh)
        }

        if raw_value == 0 {
            return SensorResult::new(raw_value as f32, SensorCode::GpioError)
        }

        const SENSITIVITY_B: f32 = 4000.0;
        const ROOM_TEMP_KELVIN: f32 = 298.15;
        const THERMISTOR_ROOM_RESISTANCE: f32 = 10000.0;

        let raw = raw_value as f32;
        let calibration_factor = 13150.0;
        let resistance = calibration_factor * (( 4095.0 - raw) / raw);

        if resistance <= 0.1 {
            return SensorResult::new(resistance, SensorCode::GpioError)
        }

        /* Use steinhart equation */
        let mut steinhart = resistance / THERMISTOR_ROOM_RESISTANCE;
        steinhart = steinhart.ln();
        steinhart /= SENSITIVITY_B;
        steinhart += 1.0 / ROOM_TEMP_KELVIN;
        steinhart = 1.0 / steinhart;

        SensorResult::new(steinhart - 273.15, SensorCode::Ok)
    }

    /// Light sensor
    /// Converts the raw ADC value from photoresistor into percentages of light.
    ///
    /// # Arguments
    /// `raw_value` - Raw reading from ADC1 converter.
    ///
    /// # Returns
    /// (`percentage` and `status code`)
    fn raw_to_light(&self, raw_value: u16) -> SensorResult<f32> {
        let raw = raw_value as f32;
        let dark_adc = 150.0;
        let light_adc = 3192.0;

        if (light_adc - dark_adc) < 1.0 {
            return SensorResult::new(0.0, SensorCode::BadCalibration)
        }

        if raw_value > 4000 {
            return SensorResult::new(raw_value as f32, SensorCode::VoltageToHigh)
        }

        let mut percentage = ((raw - dark_adc) / (light_adc - dark_adc)) * 100.0;

        if percentage > 100.0 { percentage = 100.0; }
        if percentage < 0.0 { percentage = 0.0; }

        SensorResult::new(percentage, SensorCode::Ok)
    }

    /// Tilt sensor
    /// Detects shocks
    ///
    /// # Returns
    /// `true` or `false` if detects anything
    fn is_tilted(&self) -> SensorResult<bool> {
        let is_tilt = self.hardware.tilt_pin.is_low();

        SensorResult::new(is_tilt, SensorCode::Ok)
    }

    /// Ultrasonic distance sensor
    /// Measures distance to objects up to 4 metres. Time taken for sound to
    /// be sent and received is converted into centimetres.
    ///
    /// # Returns
    /// `distance` in sentimetres
    fn measure_distance(&mut self) -> SensorResult<f32> {
        /* Clean area */
        if self.hardware.trig_pin.set_low().is_err() {
            return SensorResult::new(0.0, SensorCode::GpioError)
        }

        Ets::delay_us(2);
        if self.hardware.trig_pin.set_high().is_err() {
            return SensorResult::new(0.0, SensorCode::GpioError)
        }

        Ets::delay_us(10);
        if self.hardware.trig_pin.set_low().is_err() {
            return SensorResult::new(0.0, SensorCode::GpioError)
        }

        let mut timeout = 0;
        while self.hardware.echo_pin.is_low() {
            Ets::delay_us(1);
            timeout += 1;
            if timeout > 10000 {
                return SensorResult::new(0.0, SensorCode::HardwareTimeout) // Echo sensor does not work
            }
        }

        let start_time = Instant::now();

        /* Waiting for sound to come back  */
        timeout = 0;
        while self.hardware.echo_pin.is_high() {
            Ets::delay_us(1);
            timeout += 1;
            if timeout > 25000 {
                return SensorResult::new(0.0, SensorCode::EchoTimeout) // No obstacles ( 4 metre radius )
            }
        }

        /* duration time */
        let duration = start_time.elapsed();
        let duration_us = duration.as_micros() as f32;

        /* distance */
        let distance_cm = (duration_us * 0.0343) / 2.0;

        SensorResult::new(distance_cm, SensorCode::Ok)
    }

    /*pub fn start_monitor(&mut self) -> anyhow::Result<()> {
        let mut channel_config = AdcChannelConfig::new();
        channel_config.attenuation = DB_11;
        channel_config.calibration = Calibration::Curve;

        let mut avg_temp: Vec<f32> = vec![1.0, 1.0, 1.0, 1.0, 1.0];

        loop {
            // TEMP //
            let raw_temp = {
                let mut temp_channel = AdcChannelDriver::new(
                    &self.hardware.adc_driver,
                    &mut self.hardware.temp_pin,
                    &channel_config,
                )?;

                temp_channel.read()?
            };

            // LIGHT //
            let raw_light = {
                    let mut light_channel = AdcChannelDriver::new(
                        &self.hardware.adc_driver,
                        &mut self.hardware.light_pin,
                        &channel_config,
                    )?;

                    light_channel.read()?
                };

            // TILT //
            let tilt_status = if self.is_tilted() {
                "Stable"
            } else {
                "Tilt"
            };

            // VIBRATION //
            let mut shock_detected = false;
            let window_start = Instant::now();

            while window_start.elapsed() < Duration::from_secs(2) {
                if self.hardware.vibration_pin.is_low() {
                    shock_detected = true;
                }
            }

            Ets::delay_us(50);

            // DISTANCE //
            match self.measure_distance() {
                Some(dist) => println!("[DISTANCE] -> {:.2}cm", dist),
                None => println!("[DISTANCE] -> No obstacles"),
            }

            if shock_detected {
                println!("[VIBRATION] -> True")
            } else {
                println!("[VIBRATION] -> None")
            }

            //

            if let Some(celsius) = self.raw_to_celsius(raw_temp) {
                avg_temp.push(celsius);
                let last_5_values: f32 = avg_temp.iter().rev().take(5).copied().sum::<f32>() / 5.0;
                println!("[TEMP] -> {:.2}°C (ADC: {}) <Avarage: {:.2}>", celsius, raw_temp, last_5_values);
            }
            let light_percent = self.raw_to_light(raw_light);
            println!("[LIGHT] -> {:.2}% (ADC: {})", light_percent, raw_light);
            println!("[TILT] -> {}", tilt_status);

            thread::sleep(Duration::from_secs(2));
        }
    }*/
}
