//! # DS3231 Real-Time Clock (RTC) Driver
//!
//! A platform-agnostic Rust driver for the DS3231 Real-Time Clock, built on the `embedded-hal` ecosystem.
//! The DS3231 is a low-cost, extremely accurate I²C real-time clock (RTC) with an integrated
//! temperature-compensated crystal oscillator (TCXO) and crystal.
//!
//! ## Features
//!
//! - Both blocking and async I²C operation support
//! - Optional logging support via `log` or `defmt`
//!
//! ### Blocking Usage
//!
//! ```rust,ignore
//! use ds3231::{DS3231, Config, TimeRepresentation, SquareWaveFrequency, InterruptControl, Ocillator, Alarm1Config, Alarm2Config};
//!
//! // Create configuration
//! let config = Config {
//!     time_representation: TimeRepresentation::TwentyFourHour,
//!     square_wave_frequency: SquareWaveFrequency::Hz1,
//!     interrupt_control: InterruptControl::SquareWave,
//!     battery_backed_square_wave: false,
//!     oscillator_enable: Ocillator::Enabled,
//! };
//!
//! // Initialize device with I2C
//! let mut rtc = DS3231::new(i2c, 0x68);
//!
//! // Configure the device
//! rtc.configure(&config)?;
//!
//! // Get current date/time
//! let datetime = rtc.datetime()?;
//!
//! // Set a daily alarm for 9:30 AM
//! let alarm1 = Alarm1Config::AtTime {
//!     hours: 9,
//!     minutes: 30,
//!     seconds: 0,
//!     is_pm: None, // 24-hour mode
//! };
//! rtc.set_alarm1(&alarm1)?;
//!
//! // Set a weekly alarm for Friday at 5:00 PM
//! let alarm2 = Alarm2Config::AtTimeOnDay {
//!     hours: 5,
//!     minutes: 0,
//!     day: 6, // Friday
//!     is_pm: Some(true), // 12-hour mode
//! };
//! rtc.set_alarm2(&alarm2)?;
//! ```
//!
//! ### Async Usage
//!
//! Enable the async feature in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! ds3231 = { version = "0.1", features = ["async"] }
//! ```
//!
//! Then use with async/await:
//!
//! ```rust,ignore
//! use ds3231::{DS3231, Alarm1Config, Alarm2Config};
//!
//! // Initialize device
//! let mut rtc = DS3231::new(i2c, 0x68);
//!
//! // Configure asynchronously
//! rtc.configure(&config).await?;
//!
//! // Get current date/time asynchronously
//! let datetime = rtc.datetime().await?;
//!
//! // Set alarms asynchronously
//! let alarm1 = Alarm1Config::AtTime {
//!     hours: 9,
//!     minutes: 30,
//!     seconds: 0,
//!     is_pm: None,
//! };
//! rtc.set_alarm1(&alarm1).await?;
//!
//! let alarm2 = Alarm2Config::EveryMinute;
//! rtc.set_alarm2(&alarm2).await?;
//! ```
//!
//! ## Features
//!
//! - `async` - Enables optional async I²C support
//! - `log` - Enables logging via the `log` crate
//! - `defmt` - Enables logging via the `defmt` crate
//!
//! ## Register Map
//!
//! The driver provides access to all DS3231 registers:
//!
//! - Time/Date: seconds, minutes, hours, day, date, month, year
//! - Alarms: alarm1 (seconds to day/date), alarm2 (minutes to day/date)
//! - Control: oscillator, square wave, interrupts
//! - Status: oscillator stop, 32kHz output, busy flags
//! - Aging offset
//! - Temperature
//!
//! ## Error Handling
//!
//! The driver uses a custom error type `DS3231Error` that wraps:
//! - I²C communication errors
//! - `DateTime` validation errors
//!
//! ## Safety
//!
//! This driver uses no `unsafe` code and ensures type safety through:
//! - Strong typing for all register operations
//! - Validation of all datetime values
//! - Proper error propagation

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// MUST be the first module
mod fmt;

mod alarm;
mod datetime;
mod registers;

use chrono::NaiveDateTime;
use datetime::DS3231DateTimeError;
#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c as AsyncI2c;
use paste::paste;

use crate::datetime::DS3231DateTime;
use crate::registers::RegAddr;

// Re-export public types from alarm module
pub use crate::alarm::{Alarm1Config, Alarm2Config, AlarmError, DS3231Alarm1, DS3231Alarm2};
// Re-export public types from registers module
pub use crate::registers::{
    AgingOffset, AlarmDayDate, AlarmHours, AlarmMinutes, AlarmSeconds, Control, Date, Day,
    DayDateSelect, Hours, InterruptControl, Minutes, Month, Ocillator, Seconds,
    SquareWaveFrequency, Status, Temperature, TemperatureFraction, TimeRepresentation, Year,
};

/// Configuration for the DS3231 RTC device.
///
/// This struct contains all configurable parameters for the DS3231 device,
/// including time representation format, square wave output settings,
/// and oscillator control.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Time representation format (12-hour or 24-hour)
    pub time_representation: TimeRepresentation,
    /// Frequency of the square wave output
    pub square_wave_frequency: SquareWaveFrequency,
    /// Control mode for the interrupt output pin
    pub interrupt_control: InterruptControl,
    /// Enable square wave output when running on battery power
    pub battery_backed_square_wave: bool,
    /// Enable or disable the oscillator
    pub oscillator_enable: Ocillator,
}

/// Error type for DS3231 operations.
#[derive(Debug)]
pub enum DS3231Error<I2CE> {
    /// I2C bus error
    I2c(I2CE),
    /// `DateTime` validation or conversion error
    DateTime(DS3231DateTimeError),
    /// Alarm configuration error
    Alarm(AlarmError),
}

impl<I2CE> From<I2CE> for DS3231Error<I2CE> {
    /// Creates a `DS3231Error` from an I2C error.
    fn from(e: I2CE) -> Self {
        DS3231Error::I2c(e)
    }
}

/// DS3231 Real-Time Clock driver.
///
/// This struct provides the blocking interface to the DS3231 RTC device.
pub struct DS3231<I2C> {
    i2c: I2C,
    address: u8,
    time_representation: TimeRepresentation,
}

// Register access implementations
macro_rules! impl_register_access {
    ($(($name:ident, $regaddr:expr, $typ:ty)),+) => {
        $(
            paste! {
                #[doc = concat!("Gets the value of the ", stringify!($name), " register.")]
                #[doc = "\n\n# Returns"]
                #[doc = concat!("* `Ok(", stringify!($typ), ")` - The register value on success")]
                #[doc = "* `Err(DS3231Error)` on error"]
                #[doc = "\n\n# Errors"]
                #[doc = "Returns `DS3231Error::I2c` if there is an I2C communication error"]
                #[cfg(feature = "async")]
                pub async fn $name(&mut self) -> Result<$typ, DS3231Error<E>> {
                    let mut data = [0];
                    self.i2c
                        .write_read(self.address, &[$regaddr as u8], &mut data)
                        .await?;
                    Ok($typ(data[0]))
                }
                #[doc = concat!("Gets the value of the ", stringify!($name), " register.")]
                #[doc = "\n\n# Returns"]
                #[doc = concat!("* `Ok(", stringify!($typ), ")` - The register value on success")]
                #[doc = "* `Err(DS3231Error)` on error"]
                #[doc = "\n\n# Errors"]
                #[doc = "Returns `DS3231Error::I2c` if there is an I2C communication error"]
                #[cfg(not(feature = "async"))]
                pub fn $name(&mut self) -> Result<$typ, DS3231Error<E>> {
                    let mut data = [0];
                    self.i2c
                        .write_read(self.address, &[$regaddr as u8], &mut data)?;
                    Ok($typ(data[0]))
                }

                #[doc = concat!("Sets the value of the ", stringify!($name), " register.")]
                #[doc = "\n\n# Arguments"]
                #[doc = concat!("* `value` - The value to write to the ", stringify!($name), " register")]
                #[doc = "\n\n# Returns"]
                #[doc = "* `Ok(())` on success"]
                #[doc = "* `Err(DS3231Error)` on error"]
                #[doc = "\n\n# Errors"]
                #[doc = "Returns `DS3231Error::I2c` if there is an I2C communication error"]
                #[cfg(feature = "async")]
                pub async fn [<set_ $name>](&mut self, value: $typ) -> Result<(), DS3231Error<E>> {
                    self.i2c.write(
                        self.address,
                        &[$regaddr as u8, value.into()],
                    ).await?;
                    Ok(())
                }
                #[doc = concat!("Sets the value of the ", stringify!($name), " register.")]
                #[doc = "\n\n# Arguments"]
                #[doc = concat!("* `value` - The value to write to the ", stringify!($name), " register")]
                #[doc = "\n\n# Returns"]
                #[doc = "* `Ok(())` on success"]
                #[doc = "* `Err(DS3231Error)` on error"]
                #[doc = "\n\n# Errors"]
                #[doc = "Returns `DS3231Error::I2c` if there is an I2C communication error"]
                #[cfg(not(feature = "async"))]
                pub fn [<set_ $name>](&mut self, value: $typ) -> Result<(), DS3231Error<E>> {
                    self.i2c.write(
                        self.address,
                        &[$regaddr as u8, value.into()],
                    )?;
                    Ok(())
                }
            }
        )+
    }
}

#[maybe_async_cfg::maybe(
    sync(
        cfg(not(feature = "async")),
        self = "DS3231",
        idents(AsyncI2c(sync = "I2c"))
    ),
    async(feature = "async", keep_self)
)]
impl<I2C, E> DS3231<I2C>
where
    I2C: AsyncI2c<Error = E>,
{
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
    ///
    /// # Errors
    /// Returns `DS3231Error::I2c` if there is an I2C communication error.
    pub async fn configure(&mut self, config: &Config) -> Result<(), DS3231Error<E>> {
        debug!("DS3231: reading control register");
        let mut control = self.control().await?;
        control.set_oscillator_enable(config.oscillator_enable);
        control.set_battery_backed_square_wave(config.battery_backed_square_wave);
        control.set_square_wave_frequency(config.square_wave_frequency);
        control.set_interrupt_control(config.interrupt_control);
        debug!("DS3231: writing control: {:?}", control);
        self.set_control(control).await?;
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
    async fn read_raw_datetime(&mut self) -> Result<DS3231DateTime, DS3231Error<E>> {
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
    async fn write_raw_datetime(&mut self, datetime: DS3231DateTime) -> Result<(), DS3231Error<E>> {
        let data: [u8; 7] = (&datetime).into();
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
    ///
    /// # Errors
    /// * Returns `DS3231Error::I2c` if there is an I2C communication error
    /// * Returns `DS3231Error::DateTime` if the device returns invalid date/time data
    pub async fn datetime(&mut self) -> Result<NaiveDateTime, DS3231Error<E>> {
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
    ///
    /// # Errors
    /// * Returns `DS3231Error::I2c` if there is an I2C communication error
    /// * Returns `DS3231Error::DateTime` if the provided datetime is invalid for the device
    pub async fn set_datetime(&mut self, datetime: &NaiveDateTime) -> Result<(), DS3231Error<E>> {
        let raw = DS3231DateTime::from_datetime(datetime, self.time_representation)
            .map_err(DS3231Error::DateTime)?;
        self.write_raw_datetime(raw).await?;
        Ok(())
    }

    /// Gets the current Alarm 1 configuration from the device.
    ///
    /// # Returns
    /// * `Ok(Alarm1Config)` - The current alarm 1 configuration
    /// * `Err(DS3231Error)` on error
    ///
    /// # Errors
    /// * Returns `DS3231Error::I2c` if there is an I2C communication error
    /// * Returns `DS3231Error::Alarm` if the device contains invalid alarm register values
    pub async fn alarm1(&mut self) -> Result<Alarm1Config, DS3231Error<E>> {
        let seconds = self.alarm1_second().await?;
        let minutes = self.alarm1_minute().await?;
        let hours = self.alarm1_hour().await?;
        let day_date = self.alarm1_day_date().await?;

        let alarm = DS3231Alarm1::from_registers(seconds, minutes, hours, day_date);
        alarm.to_config().map_err(DS3231Error::Alarm)
    }

    /// Sets Alarm 1 configuration.
    ///
    /// Alarm 1 supports seconds-level precision and various matching modes.
    ///
    /// # Arguments
    /// * `config` - The alarm configuration
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(DS3231Error)` on error
    ///
    /// # Errors
    /// * Returns `DS3231Error::I2c` if there is an I2C communication error
    /// * Returns `DS3231Error::Alarm` if the provided configuration is invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ds3231::{DS3231, Alarm1Config};
    ///
    /// // Daily alarm at 9:30:00 AM (24-hour mode)
    /// let daily_alarm = Alarm1Config::AtTime {
    ///     hours: 9,
    ///     minutes: 30,
    ///     seconds: 0,
    ///     is_pm: None, // 24-hour mode
    /// };
    /// rtc.set_alarm1(&daily_alarm).await?;
    ///
    /// // Weekly alarm every Monday at 6:30:15 PM (12-hour mode)
    /// let weekly_alarm = Alarm1Config::AtTimeOnDay {
    ///     hours: 6,
    ///     minutes: 30,
    ///     seconds: 15,
    ///     day: 2, // Monday (1=Sunday, 2=Monday, etc.)
    ///     is_pm: Some(true), // 6:30 PM
    /// };
    /// rtc.set_alarm1(&weekly_alarm).await?;
    ///
    /// // Monthly alarm on the 15th at 12:00:00 PM
    /// let monthly_alarm = Alarm1Config::AtTimeOnDate {
    ///     hours: 12,
    ///     minutes: 0,
    ///     seconds: 0,
    ///     date: 15, // 15th of every month
    ///     is_pm: None, // 24-hour mode
    /// };
    /// rtc.set_alarm1(&monthly_alarm).await?;
    ///
    /// // Alarm every second (useful for testing)
    /// let frequent_alarm = Alarm1Config::EverySecond;
    /// rtc.set_alarm1(&frequent_alarm).await?;
    ///
    /// // Alarm when seconds match (every minute at 30 seconds)
    /// let seconds_alarm = Alarm1Config::AtSeconds {
    ///     seconds: 30,
    /// };
    /// rtc.set_alarm1(&seconds_alarm).await?;
    ///
    /// // Alarm when minutes and seconds match (every hour at 15:45)
    /// let minutes_seconds_alarm = Alarm1Config::AtMinutesSeconds {
    ///     minutes: 15,
    ///     seconds: 45,
    /// };
    /// rtc.set_alarm1(&minutes_seconds_alarm).await?;
    /// ```
    pub async fn set_alarm1(&mut self, config: &Alarm1Config) -> Result<(), DS3231Error<E>> {
        let alarm = DS3231Alarm1::from_config(config).map_err(DS3231Error::Alarm)?;

        self.set_alarm1_second(alarm.seconds()).await?;
        self.set_alarm1_minute(alarm.minutes()).await?;
        self.set_alarm1_hour(alarm.hours()).await?;
        self.set_alarm1_day_date(alarm.day_date()).await?;

        Ok(())
    }

    /// Gets the current Alarm 2 configuration from the device.
    ///
    /// # Returns
    /// * `Ok(Alarm2Config)` - The current alarm 2 configuration
    /// * `Err(DS3231Error)` on error
    ///
    /// # Errors
    /// * Returns `DS3231Error::I2c` if there is an I2C communication error
    /// * Returns `DS3231Error::Alarm` if the device contains invalid alarm register values
    pub async fn alarm2(&mut self) -> Result<Alarm2Config, DS3231Error<E>> {
        let minutes = self.alarm2_minute().await?;
        let hours = self.alarm2_hour().await?;
        let day_date = self.alarm2_day_date().await?;

        let alarm = DS3231Alarm2::from_registers(minutes, hours, day_date);
        alarm.to_config().map_err(DS3231Error::Alarm)
    }

    /// Sets Alarm 2 configuration.
    ///
    /// Alarm 2 has no seconds register and always triggers at 00 seconds of the matching minute.
    /// It provides minute-level precision for various matching modes.
    ///
    /// # Arguments
    /// * `config` - The alarm configuration
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(DS3231Error)` on error
    ///
    /// # Errors
    /// * Returns `DS3231Error::I2c` if there is an I2C communication error
    /// * Returns `DS3231Error::Alarm` if the provided configuration is invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ds3231::{DS3231, Alarm2Config};
    ///
    /// // Daily alarm at 9:30:00 AM (triggers at 00 seconds)
    /// let daily_alarm = Alarm2Config::AtTime {
    ///     hours: 9,
    ///     minutes: 30,
    ///     is_pm: None, // 24-hour mode
    /// };
    /// rtc.set_alarm2(&daily_alarm).await?;
    ///
    /// // Weekly alarm every Friday at 5:00:00 PM (12-hour mode)
    /// let weekly_alarm = Alarm2Config::AtTimeOnDay {
    ///     hours: 5,
    ///     minutes: 0,
    ///     day: 6, // Friday (1=Sunday, 6=Friday)
    ///     is_pm: Some(true), // 5:00 PM
    /// };
    /// rtc.set_alarm2(&weekly_alarm).await?;
    ///
    /// // Monthly alarm on the 1st at 8:15:00 AM
    /// let monthly_alarm = Alarm2Config::AtTimeOnDate {
    ///     hours: 8,
    ///     minutes: 15,
    ///     date: 1, // 1st of every month
    ///     is_pm: None, // 24-hour mode
    /// };
    /// rtc.set_alarm2(&monthly_alarm).await?;
    ///
    /// // Alarm every minute (at 00 seconds, useful for testing)
    /// let frequent_alarm = Alarm2Config::EveryMinute;
    /// rtc.set_alarm2(&frequent_alarm).await?;
    ///
    /// // Alarm when minutes match (every hour at 45:00)
    /// let minutes_alarm = Alarm2Config::AtMinutes {
    ///     minutes: 45,
    /// };
    /// rtc.set_alarm2(&minutes_alarm).await?;
    /// ```
    pub async fn set_alarm2(&mut self, config: &Alarm2Config) -> Result<(), DS3231Error<E>> {
        let alarm = DS3231Alarm2::from_config(config).map_err(DS3231Error::Alarm)?;

        self.set_alarm2_minute(alarm.minutes()).await?;
        self.set_alarm2_hour(alarm.hours()).await?;
        self.set_alarm2_day_date(alarm.day_date()).await?;

        Ok(())
    }

    impl_register_access!(
        (second, RegAddr::Seconds, Seconds),
        (minute, RegAddr::Minutes, Minutes),
        (hour, RegAddr::Hours, Hours),
        (day, RegAddr::Day, Day),
        (date, RegAddr::Date, Date),
        (month, RegAddr::Month, Month),
        (year, RegAddr::Year, Year),
        (alarm1_second, RegAddr::Alarm1Seconds, AlarmSeconds),
        (alarm1_minute, RegAddr::Alarm1Minutes, AlarmMinutes),
        (alarm1_hour, RegAddr::Alarm1Hours, AlarmHours),
        (alarm1_day_date, RegAddr::Alarm1DayDate, AlarmDayDate),
        (alarm2_minute, RegAddr::Alarm2Minutes, AlarmMinutes),
        (alarm2_hour, RegAddr::Alarm2Hours, AlarmHours),
        (alarm2_day_date, RegAddr::Alarm2DayDate, AlarmDayDate),
        (control, RegAddr::Control, Control),
        (status, RegAddr::ControlStatus, Status),
        (aging_offset, RegAddr::AgingOffset, AgingOffset),
        (temperature, RegAddr::MSBTemp, Temperature),
        (temperature_fraction, RegAddr::LSBTemp, TemperatureFraction)
    );
}

#[maybe_async_cfg::maybe(sync(cfg(not(feature = "async"))), async(feature = "async", keep_self))]
#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec;

    use super::*;
    use chrono::{Datelike, NaiveDate, Timelike};
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTrans};

    const DEVICE_ADDRESS: u8 = 0x68;

    fn setup_mock(expectations: &[I2cTrans]) -> I2cMock {
        I2cMock::new(expectations)
    }

    #[test]
    fn test_new_device() {
        let mock = setup_mock(&[]);
        let mut _dev = DS3231::new(mock, DEVICE_ADDRESS);
        // No I2C operations should happen during initialization
        _dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_read_control() {
        let expected = 0b0000_0000; // Hz1 frequency (0b00 in bits 4,3)
        let mock = setup_mock(&[I2cTrans::write_read(
            DEVICE_ADDRESS,
            vec![RegAddr::Control as u8],
            vec![expected],
        )]);
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        let control = dev.control().await.unwrap();
        assert_eq!(control.oscillator_enable(), Ocillator::Enabled);
        assert_eq!(control.square_wave_frequency(), SquareWaveFrequency::Hz1);
        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_write_control() {
        let mut control = Control::default();
        control.set_oscillator_enable(Ocillator::Enabled);
        control.set_square_wave_frequency(SquareWaveFrequency::Hz1024);

        let mock = setup_mock(&[I2cTrans::write(
            DEVICE_ADDRESS,
            vec![RegAddr::Control as u8, control.0],
        )]);
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        dev.set_control(control).await.unwrap();
        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_configure() {
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
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);
        dev.configure(&config).await.unwrap();
        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_read_datetime() {
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
        )]);
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

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_set_datetime() {
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
        )]);
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        dev.set_datetime(&dt).await.unwrap();
        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_register_operations() {
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
        ]);

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

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_read_temperature() {
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
        ]);
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        let temp = dev.temperature().await.unwrap();
        let frac = dev.temperature_fraction().await.unwrap();
        assert_eq!(temp.temperature(), 25);
        assert_eq!(frac.temperature_fraction(), 0x60);
        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_alarm_registers() {
        let mock = setup_mock(&[
            // Test alarm1 registers
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm1Seconds as u8],
                vec![0x30],
            ),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm1Minutes as u8],
                vec![0x45],
            ),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm1Hours as u8], vec![0x12]),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm1DayDate as u8],
                vec![0x15],
            ),
            // Test alarm2 registers
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm2Minutes as u8],
                vec![0x30],
            ),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm2Hours as u8], vec![0x08]),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm2DayDate as u8],
                vec![0x15],
            ),
            // Test setting alarm registers
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Seconds as u8, 0x00]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Minutes as u8, 0x15]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Hours as u8, 0x09]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1DayDate as u8, 0x10]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2Minutes as u8, 0x45]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2Hours as u8, 0x14]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2DayDate as u8, 0x25]),
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Test reading alarm registers
        let alarm1_sec = dev.alarm1_second().await.unwrap();
        assert_eq!(alarm1_sec.seconds(), 0);
        assert_eq!(alarm1_sec.ten_seconds(), 3);

        let alarm1_min = dev.alarm1_minute().await.unwrap();
        assert_eq!(alarm1_min.minutes(), 5);
        assert_eq!(alarm1_min.ten_minutes(), 4);

        let _alarm1_hour = dev.alarm1_hour().await.unwrap();
        let _alarm1_day_date = dev.alarm1_day_date().await.unwrap();

        let _alarm2_min = dev.alarm2_minute().await.unwrap();
        let _alarm2_hour = dev.alarm2_hour().await.unwrap();
        let _alarm2_day_date = dev.alarm2_day_date().await.unwrap();

        // Test setting alarm registers
        dev.set_alarm1_second(AlarmSeconds(0x00)).await.unwrap();
        dev.set_alarm1_minute(AlarmMinutes(0x15)).await.unwrap();
        dev.set_alarm1_hour(AlarmHours(0x09)).await.unwrap();
        dev.set_alarm1_day_date(AlarmDayDate(0x10)).await.unwrap();
        dev.set_alarm2_minute(AlarmMinutes(0x45)).await.unwrap();
        dev.set_alarm2_hour(AlarmHours(0x14)).await.unwrap();
        dev.set_alarm2_day_date(AlarmDayDate(0x25)).await.unwrap();

        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_status_register_flags() {
        let mock = setup_mock(&[
            // Test various status flag combinations
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::ControlStatus as u8],
                vec![0x00],
            ), // All flags clear
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::ControlStatus as u8],
                vec![0x88],
            ), // OSF and EN32kHz set
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::ControlStatus as u8],
                vec![0x07],
            ), // BSY, A2F, A1F set
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::ControlStatus as u8],
                vec![0x8F],
            ), // All flags set
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Test all flags clear
        let status = dev.status().await.unwrap();
        assert!(!status.oscillator_stop_flag());
        assert!(!status.enable_32khz_output());
        assert!(!status.busy());
        assert!(!status.alarm2_flag());
        assert!(!status.alarm1_flag());

        // Test OSF and EN32kHz set
        let status = dev.status().await.unwrap();
        assert!(status.oscillator_stop_flag());
        assert!(status.enable_32khz_output());
        assert!(!status.busy());
        assert!(!status.alarm2_flag());
        assert!(!status.alarm1_flag());

        // Test BSY, A2F, A1F set
        let status = dev.status().await.unwrap();
        assert!(!status.oscillator_stop_flag());
        assert!(!status.enable_32khz_output());
        assert!(status.busy());
        assert!(status.alarm2_flag());
        assert!(status.alarm1_flag());

        // Test all flags set
        let status = dev.status().await.unwrap();
        assert!(status.oscillator_stop_flag());
        assert!(status.enable_32khz_output());
        assert!(status.busy());
        assert!(status.alarm2_flag());
        assert!(status.alarm1_flag());

        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_alarm1_high_level_operations() {
        let mock = setup_mock(&[
            // Read alarm1 registers
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm1Seconds as u8],
                vec![0x30],
            ),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm1Minutes as u8],
                vec![0x45],
            ),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm1Hours as u8], vec![0x12]),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm1DayDate as u8],
                vec![0x15],
            ),
            // Set alarm1 configuration
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Seconds as u8, 0x00]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Minutes as u8, 0x30]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Hours as u8, 0x09]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1DayDate as u8, 0x80]),
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Test reading alarm1
        let alarm1 = dev.alarm1().await.unwrap();
        match alarm1 {
            Alarm1Config::AtTimeOnDate {
                hours,
                minutes,
                seconds,
                date,
                is_pm,
            } => {
                assert_eq!(hours, 12);
                assert_eq!(minutes, 45);
                assert_eq!(seconds, 30);
                assert_eq!(date, 15);
                assert_eq!(is_pm, None);
            }
            _ => panic!("Expected AtTimeOnDate alarm configuration"),
        }

        // Test setting alarm1
        let config = Alarm1Config::AtTime {
            hours: 9,
            minutes: 30,
            seconds: 0,
            is_pm: None,
        };
        dev.set_alarm1(&config).await.unwrap();

        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_alarm2_high_level_operations() {
        let mock = setup_mock(&[
            // Read alarm2 registers
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm2Minutes as u8],
                vec![0x45],
            ),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm2Hours as u8], vec![0x12]),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm2DayDate as u8],
                vec![0x15],
            ),
            // Set alarm2 configuration
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2Minutes as u8, 0x30]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2Hours as u8, 0x14]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2DayDate as u8, 0x80]),
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Test reading alarm2
        let alarm2 = dev.alarm2().await.unwrap();
        match alarm2 {
            Alarm2Config::AtTimeOnDate {
                hours,
                minutes,
                date,
                is_pm,
            } => {
                assert_eq!(hours, 12);
                assert_eq!(minutes, 45);
                assert_eq!(date, 15);
                assert_eq!(is_pm, None);
            }
            _ => panic!("Expected AtTimeOnDate alarm configuration"),
        }

        // Test setting alarm2
        let config = Alarm2Config::AtTime {
            hours: 14,
            minutes: 30,
            is_pm: None,
        };
        dev.set_alarm2(&config).await.unwrap();

        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_alarm_error_handling() {
        let mock = setup_mock(&[]);
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Test invalid alarm1 configuration
        let invalid_config = Alarm1Config::AtTime {
            hours: 25, // Invalid hour
            minutes: 30,
            seconds: 0,
            is_pm: None,
        };
        let result = dev.set_alarm1(&invalid_config).await;
        assert!(matches!(result, Err(DS3231Error::Alarm(_))));

        // Test invalid alarm2 configuration
        let invalid_config = Alarm2Config::AtTime {
            hours: 25, // Invalid hour
            minutes: 30,
            is_pm: None,
        };
        let result = dev.set_alarm2(&invalid_config).await;
        assert!(matches!(result, Err(DS3231Error::Alarm(_))));

        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_datetime_error_handling() {
        // Test invalid datetime conversion
        let invalid_datetime_data = [
            0x60, // Invalid seconds (60)
            0x30, // minutes
            0x15, // hours
            0x04, // day
            0x14, // date
            0x03, // month
            0x24, // year
        ];

        let mock = setup_mock(&[I2cTrans::write_read(
            DEVICE_ADDRESS,
            vec![RegAddr::Seconds as u8],
            invalid_datetime_data.to_vec(),
        )]);
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        let result = dev.datetime().await;
        assert!(matches!(result, Err(DS3231Error::DateTime(_))));

        dev.i2c.done();
    }

    #[test]
    fn test_day_date_select_conversions() {
        // Test DayDateSelect conversions
        assert_eq!(DayDateSelect::from(0), DayDateSelect::Date);
        assert_eq!(DayDateSelect::from(1), DayDateSelect::Day);
        assert_eq!(u8::from(DayDateSelect::Date), 0);
        assert_eq!(u8::from(DayDateSelect::Day), 1);
    }

    #[test]
    #[should_panic(expected = "Invalid value for DayDateSelect: 2")]
    fn test_invalid_day_date_select_conversion() {
        let _ = DayDateSelect::from(2);
    }

    #[test]
    fn test_config_debug_and_clone() {
        let config = Config {
            time_representation: TimeRepresentation::TwentyFourHour,
            square_wave_frequency: SquareWaveFrequency::Hz1,
            interrupt_control: InterruptControl::SquareWave,
            battery_backed_square_wave: false,
            oscillator_enable: Ocillator::Enabled,
        };

        // Test Debug trait
        let debug_str = alloc::format!("{:?}", config);
        assert!(debug_str.contains("TwentyFourHour"));

        // Test Clone trait
        let cloned_config = config.clone();
        assert_eq!(config, cloned_config);

        // Test Copy trait
        let copied_config = config;
        assert_eq!(config, copied_config);
    }

    #[test]
    fn test_register_bitfield_operations() {
        // Test Seconds register
        let mut seconds = Seconds::default();
        seconds.set_seconds(5);
        seconds.set_ten_seconds(3);
        assert_eq!(seconds.seconds(), 5);
        assert_eq!(seconds.ten_seconds(), 3);

        // Test Minutes register
        let mut minutes = Minutes::default();
        minutes.set_minutes(8);
        minutes.set_ten_minutes(4);
        assert_eq!(minutes.minutes(), 8);
        assert_eq!(minutes.ten_minutes(), 4);

        // Test Hours register
        let mut hours = Hours::default();
        hours.set_time_representation(TimeRepresentation::TwelveHour);
        hours.set_pm_or_twenty_hours(1);
        hours.set_ten_hours(1);
        hours.set_hours(2);
        assert_eq!(hours.time_representation(), TimeRepresentation::TwelveHour);
        assert_eq!(hours.pm_or_twenty_hours(), 1);
        assert_eq!(hours.ten_hours(), 1);
        assert_eq!(hours.hours(), 2);

        // Test Day register
        let mut day = Day::default();
        day.set_day(3);
        assert_eq!(day.day(), 3);

        // Test Date register
        let mut date = Date::default();
        date.set_date(5);
        date.set_ten_date(2);
        assert_eq!(date.date(), 5);
        assert_eq!(date.ten_date(), 2);

        // Test Month register
        let mut month = Month::default();
        month.set_month(2);
        month.set_ten_month(1);
        month.set_century(true);
        assert_eq!(month.month(), 2);
        assert_eq!(month.ten_month(), 1);
        assert!(month.century());

        // Test Year register
        let mut year = Year::default();
        year.set_year(4);
        year.set_ten_year(2);
        assert_eq!(year.year(), 4);
        assert_eq!(year.ten_year(), 2);

        // Test Control register
        let mut control = Control::default();
        control.set_oscillator_enable(Ocillator::Disabled);
        control.set_battery_backed_square_wave(true);
        control.set_convert_temperature(true);
        control.set_square_wave_frequency(SquareWaveFrequency::Hz4096);
        control.set_interrupt_control(InterruptControl::Interrupt);
        control.set_alarm2_interrupt_enable(true);
        control.set_alarm1_interrupt_enable(true);

        assert_eq!(control.oscillator_enable(), Ocillator::Disabled);
        assert!(control.battery_backed_square_wave());
        assert!(control.convert_temperature());
        assert_eq!(control.square_wave_frequency(), SquareWaveFrequency::Hz4096);
        assert_eq!(control.interrupt_control(), InterruptControl::Interrupt);
        assert!(control.alarm2_interrupt_enable());
        assert!(control.alarm1_interrupt_enable());

        // Test Status register
        let mut status = Status::default();
        status.set_oscillator_stop_flag(true);
        status.set_enable_32khz_output(true);
        status.set_busy(true);
        status.set_alarm2_flag(true);
        status.set_alarm1_flag(true);

        assert!(status.oscillator_stop_flag());
        assert!(status.enable_32khz_output());
        assert!(status.busy());
        assert!(status.alarm2_flag());
        assert!(status.alarm1_flag());

        // Test AgingOffset register
        let mut aging_offset = AgingOffset::default();
        aging_offset.set_aging_offset(-10);
        assert_eq!(aging_offset.aging_offset(), -10);

        // Test Temperature register
        let mut temperature = Temperature::default();
        temperature.set_temperature(25);
        assert_eq!(temperature.temperature(), 25);

        // Test TemperatureFraction register
        let mut temp_frac = TemperatureFraction::default();
        temp_frac.set_temperature_fraction(0x40);
        assert_eq!(temp_frac.temperature_fraction(), 0x40);
    }

    #[test]
    fn test_alarm_register_bitfield_operations() {
        // Test AlarmSeconds register
        let mut alarm_seconds = AlarmSeconds::default();
        alarm_seconds.set_alarm_mask1(true);
        alarm_seconds.set_ten_seconds(3);
        alarm_seconds.set_seconds(5);
        assert!(alarm_seconds.alarm_mask1());
        assert_eq!(alarm_seconds.ten_seconds(), 3);
        assert_eq!(alarm_seconds.seconds(), 5);

        // Test AlarmMinutes register
        let mut alarm_minutes = AlarmMinutes::default();
        alarm_minutes.set_alarm_mask2(true);
        alarm_minutes.set_ten_minutes(4);
        alarm_minutes.set_minutes(8);
        assert!(alarm_minutes.alarm_mask2());
        assert_eq!(alarm_minutes.ten_minutes(), 4);
        assert_eq!(alarm_minutes.minutes(), 8);

        // Test AlarmHours register
        let mut alarm_hours = AlarmHours::default();
        alarm_hours.set_alarm_mask3(true);
        alarm_hours.set_time_representation(TimeRepresentation::TwelveHour);
        alarm_hours.set_pm_or_twenty_hours(1);
        alarm_hours.set_ten_hours(1);
        alarm_hours.set_hours(2);
        assert!(alarm_hours.alarm_mask3());
        assert_eq!(
            alarm_hours.time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(alarm_hours.pm_or_twenty_hours(), 1);
        assert_eq!(alarm_hours.ten_hours(), 1);
        assert_eq!(alarm_hours.hours(), 2);

        // Test AlarmDayDate register
        let mut alarm_day_date = AlarmDayDate::default();
        alarm_day_date.set_alarm_mask4(true);
        alarm_day_date.set_day_date_select(DayDateSelect::Day);
        alarm_day_date.set_ten_date(2);
        alarm_day_date.set_day_or_date(5);
        assert!(alarm_day_date.alarm_mask4());
        assert_eq!(alarm_day_date.day_date_select(), DayDateSelect::Day);
        assert_eq!(alarm_day_date.ten_date(), 2);
        assert_eq!(alarm_day_date.day_or_date(), 5);
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_comprehensive_register_coverage() {
        let mock = setup_mock(&[
            // Test all register reads
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Seconds as u8], vec![0x45]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Minutes as u8], vec![0x30]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Hours as u8], vec![0x15]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Day as u8], vec![0x04]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Date as u8], vec![0x14]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Month as u8], vec![0x03]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Year as u8], vec![0x24]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Control as u8], vec![0x1C]),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::ControlStatus as u8],
                vec![0x88],
            ),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::AgingOffset as u8], vec![0x05]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::MSBTemp as u8], vec![0x19]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::LSBTemp as u8], vec![0x40]),
            // Test all register writes
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Seconds as u8, 0x30]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Minutes as u8, 0x45]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Hours as u8, 0x12]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Day as u8, 0x02]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Date as u8, 0x10]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Month as u8, 0x06]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Year as u8, 0x25]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Control as u8, 0x04]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::ControlStatus as u8, 0x00]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::AgingOffset as u8, 0x0A]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::MSBTemp as u8, 0x20]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::LSBTemp as u8, 0x80]),
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Test reading all registers
        let _seconds = dev.second().await.unwrap();
        let _minutes = dev.minute().await.unwrap();
        let _hours = dev.hour().await.unwrap();
        let _day = dev.day().await.unwrap();
        let _date = dev.date().await.unwrap();
        let _month = dev.month().await.unwrap();
        let _year = dev.year().await.unwrap();
        let _control = dev.control().await.unwrap();
        let _status = dev.status().await.unwrap();
        let _aging_offset = dev.aging_offset().await.unwrap();
        let _temperature = dev.temperature().await.unwrap();
        let _temp_fraction = dev.temperature_fraction().await.unwrap();

        // Test writing all registers
        dev.set_second(Seconds(0x30)).await.unwrap();
        dev.set_minute(Minutes(0x45)).await.unwrap();
        dev.set_hour(Hours(0x12)).await.unwrap();
        dev.set_day(Day(0x02)).await.unwrap();
        dev.set_date(Date(0x10)).await.unwrap();
        dev.set_month(Month(0x06)).await.unwrap();
        dev.set_year(Year(0x25)).await.unwrap();
        dev.set_control(Control(0x04)).await.unwrap();
        dev.set_status(Status(0x00)).await.unwrap();
        dev.set_aging_offset(AgingOffset(0x0A)).await.unwrap();
        dev.set_temperature(Temperature(0x20)).await.unwrap();
        dev.set_temperature_fraction(TemperatureFraction(0x80))
            .await
            .unwrap();

        dev.i2c.done();
    }

    #[test]
    fn test_error_type_coverage() {
        use crate::alarm::AlarmError;
        use crate::datetime::DS3231DateTimeError;

        // Test DS3231Error variants
        let datetime_error: DS3231Error<()> =
            DS3231Error::DateTime(DS3231DateTimeError::InvalidDateTime);
        assert!(matches!(datetime_error, DS3231Error::DateTime(_)));

        let alarm_error: DS3231Error<()> = DS3231Error::Alarm(AlarmError::InvalidTime("test"));
        assert!(matches!(alarm_error, DS3231Error::Alarm(_)));
    }

    #[cfg(feature = "defmt")]
    #[test]
    fn test_control_defmt_formatting() {
        // Test defmt formatting for Control register
        let mut control = Control::default();
        control.set_oscillator_enable(Ocillator::Enabled);
        control.set_battery_backed_square_wave(true);
        control.set_convert_temperature(true);
        control.set_square_wave_frequency(SquareWaveFrequency::Hz1024);
        control.set_interrupt_control(InterruptControl::Interrupt);
        control.set_alarm2_interrupt_enable(true);
        control.set_alarm1_interrupt_enable(true);

        // This test ensures the defmt::Format implementation compiles and covers the code
        // In a real embedded environment, this would produce formatted output
        let _formatted = defmt::Debug2Format(&control);
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_twelve_hour_mode_datetime() {
        // Test setting datetime in 12-hour mode
        let dt = NaiveDate::from_ymd_opt(2024, 3, 14)
            .unwrap()
            .and_hms_opt(15, 30, 0)
            .unwrap();

        let mock = setup_mock(&[
            // Configure to 12-hour mode first
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Control as u8], vec![0]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Control as u8, 0]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Hours as u8], vec![0]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Hours as u8, 0x40]), // 12-hour mode
            // Set datetime
            I2cTrans::write(
                DEVICE_ADDRESS,
                vec![
                    RegAddr::Seconds as u8,
                    0x00, // seconds
                    0x30, // minutes
                    0x63, // hours (3 PM in 12-hour mode with PM bit set)
                    0x04, // day
                    0x14, // date
                    0x03, // month
                    0x24, // year
                ],
            ),
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Configure to 12-hour mode
        let config = Config {
            time_representation: TimeRepresentation::TwelveHour,
            square_wave_frequency: SquareWaveFrequency::Hz1,
            interrupt_control: InterruptControl::SquareWave,
            battery_backed_square_wave: false,
            oscillator_enable: Ocillator::Enabled,
        };
        dev.configure(&config).await.unwrap();

        // Set datetime in 12-hour mode
        dev.set_datetime(&dt).await.unwrap();

        dev.i2c.done();
    }

    #[test]
    fn test_register_from_u8_conversions() {
        // Test TimeRepresentation conversions
        assert_eq!(
            TimeRepresentation::from(0),
            TimeRepresentation::TwentyFourHour
        );
        assert_eq!(TimeRepresentation::from(1), TimeRepresentation::TwelveHour);
        assert_eq!(u8::from(TimeRepresentation::TwentyFourHour), 0);
        assert_eq!(u8::from(TimeRepresentation::TwelveHour), 1);

        // Test Ocillator conversions
        assert_eq!(Ocillator::from(0), Ocillator::Enabled);
        assert_eq!(Ocillator::from(1), Ocillator::Disabled);
        assert_eq!(u8::from(Ocillator::Enabled), 0);
        assert_eq!(u8::from(Ocillator::Disabled), 1);

        // Test InterruptControl conversions
        assert_eq!(InterruptControl::from(0), InterruptControl::SquareWave);
        assert_eq!(InterruptControl::from(1), InterruptControl::Interrupt);
        assert_eq!(u8::from(InterruptControl::SquareWave), 0);
        assert_eq!(u8::from(InterruptControl::Interrupt), 1);

        // Test SquareWaveFrequency conversions
        assert_eq!(SquareWaveFrequency::from(0b00), SquareWaveFrequency::Hz1);
        assert_eq!(SquareWaveFrequency::from(0b01), SquareWaveFrequency::Hz1024);
        assert_eq!(SquareWaveFrequency::from(0b10), SquareWaveFrequency::Hz4096);
        assert_eq!(SquareWaveFrequency::from(0b11), SquareWaveFrequency::Hz8192);
        assert_eq!(u8::from(SquareWaveFrequency::Hz1), 0b00);
        assert_eq!(u8::from(SquareWaveFrequency::Hz1024), 0b01);
        assert_eq!(u8::from(SquareWaveFrequency::Hz4096), 0b10);
        assert_eq!(u8::from(SquareWaveFrequency::Hz8192), 0b11);
    }

    #[test]
    #[should_panic(expected = "Invalid value for TimeRepresentation: 2")]
    fn test_invalid_time_representation_conversion() {
        let _ = TimeRepresentation::from(2);
    }

    #[test]
    #[should_panic(expected = "Invalid value for Ocillator: 2")]
    fn test_invalid_oscillator_conversion() {
        let _ = Ocillator::from(2);
    }

    #[test]
    #[should_panic(expected = "Invalid value for InterruptControl: 2")]
    fn test_invalid_interrupt_control_conversion() {
        let _ = InterruptControl::from(2);
    }

    #[test]
    #[should_panic(expected = "Invalid value for SquareWaveFrequency: 4")]
    fn test_invalid_square_wave_frequency_conversion() {
        let _ = SquareWaveFrequency::from(4);
    }

    #[test]
    fn test_error_conversions() {
        // Test DS3231Error::from for I2C errors
        #[derive(Debug, PartialEq)]
        struct MockI2cError;

        let i2c_error = MockI2cError;
        let ds3231_error = DS3231Error::from(i2c_error);
        assert!(matches!(ds3231_error, DS3231Error::I2c(MockI2cError)));
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_individual_registers() {
        let mock = setup_mock(&[
            // Test all register reads
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Day as u8], vec![0x04]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Date as u8], vec![0x15]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Month as u8], vec![0x03]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Year as u8], vec![0x24]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::AgingOffset as u8], vec![0x05]),
            // Test all register writes
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Day as u8, 0x02]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Date as u8, 0x10]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Month as u8, 0x06]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Year as u8, 0x25]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::AgingOffset as u8, 0x0A]),
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Test reading all individual registers
        let day = dev.day().await.unwrap();
        assert_eq!(day.day(), 4);

        let date = dev.date().await.unwrap();
        assert_eq!(date.date(), 5);
        assert_eq!(date.ten_date(), 1);

        let month = dev.month().await.unwrap();
        assert_eq!(month.month(), 3);
        assert_eq!(month.ten_month(), 0);
        assert!(!month.century());

        let year = dev.year().await.unwrap();
        assert_eq!(year.year(), 4);
        assert_eq!(year.ten_year(), 2);

        let aging_offset = dev.aging_offset().await.unwrap();
        assert_eq!(aging_offset.aging_offset(), 5);

        // Test writing all individual registers
        dev.set_day(Day(0x02)).await.unwrap();
        dev.set_date(Date(0x10)).await.unwrap();
        dev.set_month(Month(0x06)).await.unwrap();
        dev.set_year(Year(0x25)).await.unwrap();
        dev.set_aging_offset(AgingOffset(0x0A)).await.unwrap();

        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_twelve_hour_mode() {
        let config = Config {
            time_representation: TimeRepresentation::TwelveHour,
            square_wave_frequency: SquareWaveFrequency::Hz1,
            interrupt_control: InterruptControl::SquareWave,
            battery_backed_square_wave: false,
            oscillator_enable: Ocillator::Enabled,
        };

        let mock = setup_mock(&[
            // Read control register
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Control as u8], vec![0]),
            // Write control register
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Control as u8, 0]),
            // Read hours register
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Hours as u8], vec![0]),
            // Write hours register with 12-hour mode bit set
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Hours as u8, 0x40]), // Bit 6 set for 12-hour mode
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);
        dev.configure(&config).await.unwrap();
        assert_eq!(dev.time_representation, TimeRepresentation::TwelveHour);
        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_read_alarm_mask_bits_and_dydt() {
        let mock = setup_mock(&[
            // Test reading alarm registers with mask bits set
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm1Seconds as u8],
                vec![0x80], // A1M1 mask bit set
            ),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm1Minutes as u8],
                vec![0x85], // A1M2 mask bit set + 5 minutes
            ),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm1Hours as u8],
                vec![0x89], // A1M3 mask bit set + 9 hours
            ),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm1DayDate as u8],
                vec![0xC3], // A1M4=1, DY/DT=1, day=3
            ),
            I2cTrans::write_read(
                DEVICE_ADDRESS,
                vec![RegAddr::Alarm2DayDate as u8],
                vec![0x15], // A2M4 clear + DY/DT clear + date 15
            ),
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Test reading alarm registers with mask bits
        let alarm1_sec = dev.alarm1_second().await.unwrap();
        assert!(alarm1_sec.alarm_mask1()); // Mask bit should be set
        assert_eq!(alarm1_sec.seconds(), 0);
        assert_eq!(alarm1_sec.ten_seconds(), 0);

        let alarm1_min = dev.alarm1_minute().await.unwrap();
        assert!(alarm1_min.alarm_mask2()); // Mask bit should be set
        assert_eq!(alarm1_min.minutes(), 5);
        assert_eq!(alarm1_min.ten_minutes(), 0);

        let alarm1_hour = dev.alarm1_hour().await.unwrap();
        assert!(alarm1_hour.alarm_mask3()); // Mask bit should be set
        assert_eq!(alarm1_hour.hours(), 9);
        assert_eq!(alarm1_hour.ten_hours(), 0);

        let alarm1_day_date = dev.alarm1_day_date().await.unwrap();
        assert!(alarm1_day_date.alarm_mask4()); // Mask bit should be set
        assert_eq!(alarm1_day_date.day_date_select(), DayDateSelect::Day); // DY/DT should be set (day mode) for 0xC3
        assert_eq!(alarm1_day_date.day_or_date(), 3); // Day 3 (0xC3 & 0x0F = 3)

        let alarm2_day_date = dev.alarm2_day_date().await.unwrap();
        assert!(!alarm2_day_date.alarm_mask4()); // Mask bit should be clear
        assert_eq!(alarm2_day_date.day_date_select(), DayDateSelect::Date); // DY/DT should be clear (date mode)
        assert_eq!(alarm2_day_date.day_or_date(), 5); // Date 5
        assert_eq!(alarm2_day_date.ten_date(), 1); // Ten date 1

        dev.i2c.done();
    }

    #[cfg_attr(feature = "async", tokio::test)]
    #[cfg_attr(not(feature = "async"), test)]
    async fn test_write_alarm_mask_bits_and_dydt() {
        let mock = setup_mock(&[
            // Test writing alarm registers with mask bits
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Seconds as u8, 0x80]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Minutes as u8, 0x85]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Hours as u8, 0x89]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1DayDate as u8, 0xC3]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2Minutes as u8, 0x45]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2Hours as u8, 0x14]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2DayDate as u8, 0x15]),
        ]);

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        // Test writing alarm registers with mask bits
        let mut alarm1_sec = AlarmSeconds(0x00);
        alarm1_sec.set_alarm_mask1(true);
        dev.set_alarm1_second(alarm1_sec).await.unwrap();

        let mut alarm1_min = AlarmMinutes(0x05);
        alarm1_min.set_alarm_mask2(true);
        dev.set_alarm1_minute(alarm1_min).await.unwrap();

        let mut alarm1_hour = AlarmHours(0x09);
        alarm1_hour.set_alarm_mask3(true);
        dev.set_alarm1_hour(alarm1_hour).await.unwrap();

        let mut alarm1_day_date = AlarmDayDate(0x03);
        alarm1_day_date.set_alarm_mask4(true);
        alarm1_day_date.set_day_date_select(DayDateSelect::Day); // Set to day mode
        dev.set_alarm1_day_date(alarm1_day_date).await.unwrap();

        let alarm2_min = AlarmMinutes(0x45);
        dev.set_alarm2_minute(alarm2_min).await.unwrap();

        let alarm2_hour = AlarmHours(0x14);
        dev.set_alarm2_hour(alarm2_hour).await.unwrap();

        let mut alarm2_day_date = AlarmDayDate(0x15);
        alarm2_day_date.set_alarm_mask4(false);
        alarm2_day_date.set_day_date_select(DayDateSelect::Date); // Set to date mode
        dev.set_alarm2_day_date(alarm2_day_date).await.unwrap();

        dev.i2c.done();
    }

    #[test]
    fn test_ds3231_error_display_coverage() {
        // Test DS3231Error Debug implementation for different error types
        use crate::alarm::AlarmError;
        use crate::datetime::DS3231DateTimeError;

        let i2c_error: DS3231Error<&str> = DS3231Error::I2c("I2C communication failed");
        let debug_str = alloc::format!("{:?}", i2c_error);
        assert!(debug_str.contains("I2c"));

        let datetime_error: DS3231Error<()> =
            DS3231Error::DateTime(DS3231DateTimeError::InvalidDateTime);
        let debug_str = alloc::format!("{:?}", datetime_error);
        assert!(debug_str.contains("DateTime"));

        let alarm_error: DS3231Error<()> = DS3231Error::Alarm(AlarmError::InvalidTime("test"));
        let debug_str = alloc::format!("{:?}", alarm_error);
        assert!(debug_str.contains("Alarm"));
    }
}
