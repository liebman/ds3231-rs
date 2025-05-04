//! Async implementation of the DS3231 driver.
//!
//! This module provides an async interface to the DS3231 RTC device using
//! `embedded-hal-async` traits. It is only available when the `async` feature
//! is enabled.
//!
//! # Example
//!
//! ```rust,ignore
//! use ds3231::asynch::DS3231;
//!
//! // Initialize device
//! let mut rtc = DS3231::new(i2c, 0x68);
//!
//! // Configure asynchronously
//! rtc.configure(&config).await?;
//!
//! // Get current date/time asynchronously
//! let datetime = rtc.datetime().await?;
//! ```

use chrono::NaiveDateTime;
use embedded_hal_async::i2c::I2c;
use paste::paste;

use crate::{
    AgingOffset, Config, Control, DS3231DateTime, DS3231Error, Date, Day, Hours, Minutes, Month,
    RegAddr, Seconds, Status, Temperature, TemperatureFraction, TimeRepresentation, Year,
};

/// DS3231 Real-Time Clock async driver.
///
/// This struct provides the async interface to the DS3231 RTC device.
/// It supports async I2C operations through the `embedded-hal-async` traits.
pub struct DS3231<I2C: I2c> {
    i2c: I2C,
    address: u8,
    time_representation: TimeRepresentation,
}

impl<I2C: I2c> DS3231<I2C> {
    /// Creates a new DS3231 async driver instance.
    ///
    /// # Arguments
    /// * `i2c` - The async I2C bus implementation
    /// * `address` - The I2C address of the device (typically 0x68)
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            time_representation: TimeRepresentation::TwentyFourHour,
        }
    }

    /// Configures the device according to the provided configuration.
    ///
    /// # Arguments
    /// * `config` - The configuration to apply
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(DS3231Error)` on error
    pub async fn configure(&mut self, config: &Config) -> Result<(), DS3231Error<I2C::Error>> {
        #[cfg(any(feature = "log", feature = "defmt"))]
        debug!("DS3231: reading control register");
        let mut control = self.control().await?;
        control.set_oscillator_enable(config.oscillator_enable);
        control.set_battery_backed_square_wave(config.battery_backed_square_wave);
        control.set_square_wave_frequency(config.square_wave_frequency);
        control.set_interrupt_control(config.interrupt_control);
        #[cfg(any(feature = "log", feature = "defmt"))]
        debug!("DS3231: writing control: {:?}", control);
        self.set_control(control).await?;
        #[cfg(any(feature = "log", feature = "defmt"))]
        debug!("DS3231: reading hours register");
        let mut hours = self.hour().await?;
        hours.set_time_representation(config.time_representation);
        self.set_hour(hours).await?;
        self.time_representation = config.time_representation;
        Ok(())
    }

    /// Reads the raw datetime registers from the device.
    ///
    /// # Returns
    /// * `Ok(DS3231DateTime)` - The raw datetime values on success
    /// * `Err(DS3231Error)` on error
    async fn read_raw_datetime(&mut self) -> Result<DS3231DateTime, DS3231Error<I2C::Error>> {
        let mut data = [0; 7];
        self.i2c
            .write_read(self.address, &[RegAddr::Seconds as u8], &mut data)
            .await?;
        Ok(data.into())
    }

    /// Writes raw datetime values to the device registers.
    ///
    /// # Arguments
    /// * `datetime` - The raw datetime values to write
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(DS3231Error)` on error
    async fn write_raw_datetime(
        &mut self,
        datetime: &DS3231DateTime,
    ) -> Result<(), DS3231Error<I2C::Error>> {
        let data: [u8; 7] = datetime.into();
        self.i2c
            .write(
                self.address,
                &[
                    RegAddr::Seconds as u8,
                    data[0],
                    data[1],
                    data[2],
                    data[3],
                    data[4],
                    data[5],
                    data[6],
                ],
            )
            .await?;
        Ok(())
    }

    /// Gets the current date and time from the device.
    ///
    /// # Returns
    /// * `Ok(NaiveDateTime)` - The current date and time
    /// * `Err(DS3231Error)` on error
    pub async fn datetime(&mut self) -> Result<NaiveDateTime, DS3231Error<I2C::Error>> {
        let raw = self.read_raw_datetime().await?;
        raw.into_datetime().map_err(DS3231Error::DateTime)
    }

    /// Sets the current date and time on the device.
    ///
    /// # Arguments
    /// * `datetime` - The date and time to set
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(DS3231Error)` on error
    pub async fn set_datetime(
        &mut self,
        datetime: &NaiveDateTime,
    ) -> Result<(), DS3231Error<I2C::Error>> {
        let raw = DS3231DateTime::from_datetime(datetime, self.time_representation)
            .map_err(DS3231Error::DateTime)?;
        self.write_raw_datetime(&raw).await?;
        Ok(())
    }
}

// Register access implementations
macro_rules! impl_register_access {
    ($(($name:ident, $regaddr:expr, $typ:ty)),+) => {
        impl<I2C: I2c> DS3231<I2C> {
            $(
                paste! {
                    #[doc = concat!("Gets the value of the ", stringify!($name), " register.")]
                    #[doc = "\n\n# Returns"]
                    #[doc = concat!("* `Ok(", stringify!($typ), ")` - The register value on success")]
                    #[doc = "* `Err(DS3231Error)` on error"]
                    pub async fn $name(&mut self) -> Result<$typ, DS3231Error<I2C::Error>> {
                        let mut data = [0];
                        self.i2c
                            .write_read(self.address, &[$regaddr as u8], &mut data)
                            .await?;
                        Ok($typ(data[0]))
                    }

                    #[doc = concat!("Sets the value of the ", stringify!($name), " register.")]
                    #[doc = "\n\n# Arguments"]
                    #[doc = concat!("* `value` - The value to write to the ", stringify!($name), " register")]
                    #[doc = "\n\n# Returns"]
                    #[doc = "* `Ok(())` on success"]
                    #[doc = "* `Err(DS3231Error)` on error"]
                    pub async fn [<set_ $name>](&mut self, value: $typ) -> Result<(), DS3231Error<I2C::Error>> {
                        self.i2c.write(
                            self.address,
                            &[$regaddr as u8, value.into()],
                        ).await?;
                        Ok(())
                    }
                }
            )+
        }
    }
}

impl_register_access!(
    (second, RegAddr::Seconds, Seconds),
    (minute, RegAddr::Minutes, Minutes),
    (hour, RegAddr::Hours, Hours),
    (day, RegAddr::Day, Day),
    (date, RegAddr::Date, Date),
    (month, RegAddr::Month, Month),
    (year, RegAddr::Year, Year),
    (alarm1_second, RegAddr::Alarm1Seconds, Seconds),
    (alarm1_minute, RegAddr::Alarm1Minutes, Minutes),
    (alarm1_hour, RegAddr::Alarm1Hours, Hours),
    (alarm1_day_date, RegAddr::Alarm1DayDate, Date),
    (alarm2_minute, RegAddr::Alarm2Minutes, Minutes),
    (alarm2_hour, RegAddr::Alarm2Hours, Hours),
    (alarm2_day_date, RegAddr::Alarm2DayDate, Date),
    (control, RegAddr::Control, Control),
    (status, RegAddr::ControlStatus, Status),
    (aging_offset, RegAddr::AgingOffset, AgingOffset),
    (temperature, RegAddr::MSBTemp, Temperature),
    (temperature_fraction, RegAddr::LSBTemp, TemperatureFraction)
);

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use crate::{InterruptControl, Ocillator, SquareWaveFrequency};
    use alloc::vec;
    use chrono::{Datelike, NaiveDate, Timelike};
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTrans};

    const DEVICE_ADDRESS: u8 = 0x68;

    async fn setup_mock(expectations: &[I2cTrans]) -> I2cMock {
        I2cMock::new(expectations)
    }

    #[tokio::test]
    async fn test_async_read_control() {
        let expected = 0b0000_0000; // Hz1 frequency (0b00 in bits 4,3)
        let mock = setup_mock(&[I2cTrans::write_read(
            DEVICE_ADDRESS,
            vec![RegAddr::Control as u8],
            vec![expected],
        )])
        .await;
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        let control = dev.control().await.unwrap();
        assert_eq!(control.oscillator_enable(), Ocillator::Enabled);
        assert_eq!(control.square_wave_frequency(), SquareWaveFrequency::Hz1);
        dev.i2c.done();
    }

    #[tokio::test]
    async fn test_async_configure() {
        let config = Config {
            time_representation: TimeRepresentation::TwentyFourHour,
            square_wave_frequency: SquareWaveFrequency::Hz1,
            interrupt_control: InterruptControl::SquareWave,
            battery_backed_square_wave: false,
            oscillator_enable: Ocillator::Enabled,
        };

        let mock = setup_mock(&[
            // Read control register
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Control as u8], vec![0]),
            // Write control register with Hz1 frequency (0b00 in bits 4,3)
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Control as u8, 0b0000_0000]),
            // Read hours register
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Hours as u8], vec![0]),
            // Write hours register
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Hours as u8, 0]),
        ])
        .await;

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);
        dev.configure(&config).await.unwrap();
        dev.i2c.done();
    }

    #[tokio::test]
    async fn test_async_read_datetime() {
        // 2024-03-14 15:30:00
        let datetime_registers = [
            0x00, // seconds
            0x30, // minutes
            0x15, // hours (24-hour mode)
            0x04, // day (Thursday)
            0x14, // date
            0x03, // month
            0x24, // year
        ];

        let mock = setup_mock(&[I2cTrans::write_read(
            DEVICE_ADDRESS,
            vec![RegAddr::Seconds as u8],
            datetime_registers.to_vec(),
        )])
        .await;
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        let dt = dev.datetime().await.unwrap();
        assert_eq!(dt.hour(), 15);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 0);
        assert_eq!(dt.day(), 14);
        assert_eq!(dt.month(), 3);
        assert_eq!(dt.year(), 2024);
        dev.i2c.done();
    }

    #[tokio::test]
    async fn test_async_set_datetime() {
        let dt = NaiveDate::from_ymd_opt(2024, 3, 14)
            .unwrap()
            .and_hms_opt(15, 30, 0)
            .unwrap();

        let mock = setup_mock(&[I2cTrans::write(
            DEVICE_ADDRESS,
            vec![
                RegAddr::Seconds as u8,
                0x00, // seconds
                0x30, // minutes (BCD for 30)
                0x15, // hours (BCD for 15 in 24-hour mode)
                0x04, // day (Thursday)
                0x14, // date
                0x03, // month
                0x24, // year
            ],
        )])
        .await;
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        dev.set_datetime(&dt).await.unwrap();
        dev.i2c.done();
    }

    #[tokio::test]
    async fn test_async_register_operations() {
        let mock = setup_mock(&[
            // Test second register
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Seconds as u8], vec![0x45]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Seconds as u8, 0x30]),
            // Test minute register
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Minutes as u8], vec![0x30]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Minutes as u8, 0x45]),
            // Test status register
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::ControlStatus as u8],
                vec![0x80],
            ),
        ])
        .await;

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Test seconds
        let seconds = dev.second().await.unwrap();
        assert_eq!(seconds.seconds(), 5);
        assert_eq!(seconds.ten_seconds(), 4);
        dev.set_second(Seconds(0x30)).await.unwrap();

        // Test minutes
        let minutes = dev.minute().await.unwrap();
        assert_eq!(minutes.minutes(), 0);
        assert_eq!(minutes.ten_minutes(), 3);
        dev.set_minute(Minutes(0x45)).await.unwrap();

        // Test status
        let status = dev.status().await.unwrap();
        assert!(status.oscillator_stop_flag());

        dev.i2c.done();
    }

    #[tokio::test]
    async fn test_async_read_temperature() {
        // Temperature value: 25°C (0x19) with fraction 0x60 (0.375°C)
        let expected_msb = 0x19; // 25°C
        let expected_lsb = 0x60; // 0.375°C

        let mock = setup_mock(&[
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::MSBTemp as u8],
                vec![expected_msb],
            ),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::LSBTemp as u8],
                vec![expected_lsb],
            ),
        ])
        .await;
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        let temp = dev.temperature().await.unwrap();
        let frac = dev.temperature_fraction().await.unwrap();
        assert_eq!(temp.temperature(), 25);
        assert_eq!(frac.temperature_fraction(), 0x60);
        dev.i2c.done();
    }
}
