use anyhow::Result;

use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

use esp_idf_hal::adc::oneshot::AdcChannelDriver;
use esp_idf_hal::adc::oneshot::config::{AdcChannelConfig, Calibration};
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::delay::{Ets, FreeRtos};

use embedded_hal::digital::v2::OutputPin as _;

use crate::config::{SensorsConfig, LiveMeasurements};
use crate::database::repository::SensorRepository;
use crate::exceptions::{SensorResult, SensorCode};


pub struct SystemMonitor {
    hardware: SensorsConfig,
    shared_data: Arc<Mutex<LiveMeasurements>>,
    db_repository: Arc<SensorRepository>,
}

impl<T> SensorResult<T> {
    fn new(value: T, code: SensorCode) -> Self {
        Self { value, code }
    }
}

impl SystemMonitor {
    pub fn new(
        hardware: SensorsConfig,
        shared_data: Arc<Mutex<LiveMeasurements>>,
        db_repository: Arc<SensorRepository>,
    ) -> Self {
        Self { hardware, shared_data, db_repository }
    }

    /// Temperature sensos
    /// Converts the raw ADC value from thermistor into degrees Celsius.
    ///
    /// # Arguments
    /// * `raw_value` - Raw reading from ADC1 converter (1 - 4095)
    ///
    /// # Returns
    /// * (`value` and `status code`)
    fn raw_to_celsius(raw_value: u16) -> SensorResult<f32> {
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
    fn raw_to_light(raw_value: u16) -> SensorResult<f32> {
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

        let wait_time = Instant::now();
        while self.hardware.echo_pin.is_low() {
            if wait_time.elapsed().as_micros() > 10000 {
                return SensorResult::new(400.0, SensorCode::HardwareTimeout);
            }
        }

        /* Waiting for sound to come back */
        let echo_start = Instant::now();
        while self.hardware.echo_pin.is_high() {
            if echo_start.elapsed().as_micros() > 25000 {
                return SensorResult::new(400.0, SensorCode::EchoTimeout);
            }
        }

        /* duration time */
        let duration_us = echo_start.elapsed().as_micros() as f32;

        /* distance */
        let distance_cm = (duration_us * 0.0343) / 2.0;

        SensorResult::new(distance_cm, SensorCode::Ok)
    }

    pub fn start_monitor(&mut self) -> Result<()> {
        let mut channel_config = AdcChannelConfig::new();
        channel_config.attenuation = DB_11;
        channel_config.calibration = Calibration::Curve;

        let mut temp_channel = AdcChannelDriver::new(
            &self.hardware.adc_driver,
            &mut self.hardware.temp_pin,
            &channel_config,
        )?;

        let mut light_channel = AdcChannelDriver::new(
            &self.hardware.adc_driver,
            &mut self.hardware.light_pin,
            &channel_config,
        )?;

        let mut last_send_time = Instant::now();
        let mut shock_detected = false;
        let mut is_alarm_active = [false; 6];

        loop {
            if self.hardware.vibration_pin.is_low() {
                shock_detected = true;
            }

            if last_send_time.elapsed() >= Duration::from_secs(1) {
                // TEMP
                let raw_temp = temp_channel.read()?;
                let temp_result = Self::raw_to_celsius(raw_temp);
                let temp_celsius = if temp_result.code == SensorCode::Ok { temp_result.value } else { 0.0 };


                // LIGHT
                let raw_light = light_channel.read()?;
                let light_percent = Self::raw_to_light(raw_light);
                let light_percent_val = if light_percent.code == SensorCode::Ok { light_percent.value } else { 0.0 };


                // TILT
                let tilt_status_bool = self.hardware.tilt_pin.is_low();

                let dist_result = {
                    if self.hardware.trig_pin.set_low().is_err() {
                        SensorResult::new(0.0, SensorCode::GpioError)
                    } else {
                        Ets::delay_us(2);
                        if self.hardware.trig_pin.set_high().is_err() {
                            SensorResult::new(0.0, SensorCode::GpioError)
                        } else {
                            Ets::delay_us(10);
                            if self.hardware.trig_pin.set_low().is_err() {
                                SensorResult::new(0.0, SensorCode::GpioError)
                            } else {
                                let wait_time = Instant::now();
                                let mut timeout_err = false;

                                while self.hardware.echo_pin.is_low() {
                                    if wait_time.elapsed().as_micros() > 10000 {
                                        timeout_err = true;
                                        break;
                                    }
                                }

                                if timeout_err {
                                    SensorResult::new(400.0, SensorCode::HardwareTimeout)
                                } else {
                                    let echo_start = Instant::now();
                                    let mut echo_err = false;

                                    while self.hardware.echo_pin.is_high() {
                                        // Max 25000us = ~4m range
                                        if echo_start.elapsed().as_micros() > 25000 {
                                            echo_err = true;
                                            break;
                                        }
                                    }

                                    if echo_err {
                                        SensorResult::new(400.0, SensorCode::EchoTimeout)
                                    } else {
                                        let duration_us = echo_start.elapsed().as_micros() as f32;
                                        SensorResult::new((duration_us * 0.0343) / 2.0, SensorCode::Ok)
                                    }
                                }
                            }
                        }
                    }
                };
                let distance_cm = if dist_result.code == SensorCode::Ok { dist_result.value } else { -1.0 };
                println!("[DISTANSE] -> {}cm", distance_cm);

                // Save data to send
                if let Ok(mut data) = self.shared_data.lock() {
                    data.temperature = temp_celsius;
                    data.light_percent = light_percent_val;
                    data.is_tilted = !tilt_status_bool;
                    data.shock_detected = shock_detected; // Zapisujemy status wstrząsów z całych 2 sekund
                    data.distance_cm = distance_cm;
                }

                let tilt_var: f32 = if tilt_status_bool { 0.0 } else { 1.0 };
                let vibe_var: f32 = if shock_detected { 1.0 } else { 0.0 };

                let _ = self.db_repository.update_all_sensors_batch(
                    temp_celsius,
                    light_percent_val,
                    tilt_var,
                    vibe_var,
                    distance_cm,
                );

                if let Ok(sensors) = self.db_repository.get_all_sensors() {
                    for s in sensors {
                        let id = s.id as usize;
                        let mut currencly_breached = false;

                        match id {
                            1 => if temp_celsius <= s.min_threshold || temp_celsius >= s.max_threshold { currencly_breached = true},
                            2 => if light_percent_val <= s.min_threshold || light_percent_val >= s.max_threshold{ currencly_breached = true },
                            3 => if !tilt_status_bool { currencly_breached = true },
                            4 => if shock_detected { currencly_breached = true },
                            5 => if distance_cm > 0.0 && distance_cm <= s.min_threshold { currencly_breached= true },
                            _ => {}
                        }

                        if currencly_breached && !is_alarm_active[id] {
                            is_alarm_active[id] = true;
                            let _ = self.db_repository.trigger_alarm(s.id, true);
                            println!("[ALARM] => Alarm to sensor: {}", s.id);
                        }

                        else if !currencly_breached && is_alarm_active[id] {
                            is_alarm_active[id] = false;
                            let _ = self.db_repository.trigger_alarm(s.id, false);
                            println!("[ALARM] => Alarm cancelled to sensor: {}", s.id);
                        }
                    }
                }


                // Reset counters to 2 sec
                shock_detected = false;
                last_send_time = Instant::now();
            }

            FreeRtos::delay_ms(10);
        }
    }
}
