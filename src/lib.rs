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
//! use ds3231::{DS3231, Config, TimeRepresentation, SquareWaveFrequency, InterruptControl, Ocillator};
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

mod datetime;

use bitfield::bitfield;
use chrono::NaiveDateTime;
use datetime::DS3231DateTimeError;
#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c as AsyncI2c;
use paste::paste;

use crate::datetime::DS3231DateTime;

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

/// Register addresses for the DS3231 RTC.
#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum RegAddr {
    /// Seconds register (0-59)
    Seconds = 0x00,
    /// Minutes register (0-59)
    Minutes = 0x01,
    /// Hours register (1-12 + AM/PM or 0-23)
    Hours = 0x02,
    /// Day register (1-7)
    Day = 0x03,
    /// Date register (1-31)
    Date = 0x04,
    /// Month register (1-12)
    Month = 0x05,
    /// Year register (0-99)
    Year = 0x06,
    /// Alarm 1 seconds register
    Alarm1Seconds = 0x07,
    /// Alarm 1 minutes register
    Alarm1Minutes = 0x08,
    /// Alarm 1 hours register
    Alarm1Hours = 0x09,
    /// Alarm 1 day/date register
    Alarm1DayDate = 0x0A,
    /// Alarm 2 minutes register
    Alarm2Minutes = 0x0B,
    /// Alarm 2 hours register
    Alarm2Hours = 0x0C,
    /// Alarm 2 day/date register
    Alarm2DayDate = 0x0D,
    /// Control register
    Control = 0x0E,
    /// Control/Status register
    ControlStatus = 0x0F,
    /// Aging offset register
    AgingOffset = 0x10,
    /// Temperature MSB register
    MSBTemp = 0x11,
    /// Temperature LSB register
    LSBTemp = 0x12,
}

/// Time representation format for the DS3231.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TimeRepresentation {
    /// 24-hour format (0-23)
    TwentyFourHour = 0,
    /// 12-hour format (1-12 + AM/PM)
    TwelveHour = 1,
}
impl From<u8> for TimeRepresentation {
    /// Creates a `TimeRepresentation` from a raw register value.
    ///
    /// # Panics
    /// Panics if the value is not 0 or 1.
    fn from(v: u8) -> Self {
        match v {
            0 => TimeRepresentation::TwentyFourHour,
            1 => TimeRepresentation::TwelveHour,
            _ => panic!("Invalid value for TimeRepresentation: {}", v),
        }
    }
}
impl From<TimeRepresentation> for u8 {
    /// Converts a `TimeRepresentation` to its raw register value.
    fn from(v: TimeRepresentation) -> Self {
        v as u8
    }
}

/// Oscillator control for the DS3231.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Ocillator {
    /// Oscillator is enabled
    Enabled = 0,
    /// Oscillator is disabled
    Disabled = 1,
}
impl From<u8> for Ocillator {
    /// Creates an Ocillator from a raw register value.
    ///
    /// # Panics
    /// Panics if the value is not 0 or 1.
    fn from(v: u8) -> Self {
        match v {
            0 => Ocillator::Enabled,
            1 => Ocillator::Disabled,
            _ => panic!("Invalid value for Ocillator: {}", v),
        }
    }
}
impl From<Ocillator> for u8 {
    /// Converts an Ocillator to its raw register value.
    fn from(v: Ocillator) -> Self {
        v as u8
    }
}

/// Interrupt control mode for the DS3231.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InterruptControl {
    /// Output square wave on INT/SQW pin
    SquareWave = 0,
    /// Output interrupt signal on INT/SQW pin
    Interrupt = 1,
}
impl From<u8> for InterruptControl {
    /// Creates an `InterruptControl` from a raw register value.
    ///
    /// # Panics
    /// Panics if the value is not 0 or 1.
    fn from(v: u8) -> Self {
        match v {
            0 => InterruptControl::SquareWave,
            1 => InterruptControl::Interrupt,
            _ => panic!("Invalid value for InterruptControl: {}", v),
        }
    }
}
impl From<InterruptControl> for u8 {
    /// Converts an `InterruptControl` to its raw register value.
    fn from(v: InterruptControl) -> Self {
        v as u8
    }
}

/// Square wave output frequency options.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SquareWaveFrequency {
    /// 1 Hz square wave output
    Hz1 = 0b00,
    /// 1.024 kHz square wave output
    Hz1024 = 0b01,
    /// 4.096 kHz square wave output
    Hz4096 = 0b10,
    /// 8.192 kHz square wave output
    Hz8192 = 0b11,
}
impl From<u8> for SquareWaveFrequency {
    /// Creates a `SquareWaveFrequency` from a raw register value.
    ///
    /// # Panics
    /// Panics if the value is not 0b00, 0b01, 0b10, or 0b11.
    fn from(v: u8) -> Self {
        match v {
            0b00 => SquareWaveFrequency::Hz1,
            0b01 => SquareWaveFrequency::Hz1024,
            0b10 => SquareWaveFrequency::Hz4096,
            0b11 => SquareWaveFrequency::Hz8192,
            _ => panic!("Invalid value for SquareWaveFrequency: {}", v),
        }
    }
}
impl From<SquareWaveFrequency> for u8 {
    /// Converts a `SquareWaveFrequency` to its raw register value.
    fn from(v: SquareWaveFrequency) -> Self {
        v as u8
    }
}

/// Error type for DS3231 operations.
#[derive(Debug)]
pub enum DS3231Error<I2CE> {
    /// I2C bus error
    I2c(I2CE),
    /// `DateTime` validation or conversion error
    DateTime(DS3231DateTimeError),
}

impl<I2CE> From<I2CE> for DS3231Error<I2CE> {
    /// Creates a `DS3231Error` from an I2C error.
    fn from(e: I2CE) -> Self {
        DS3231Error::I2c(e)
    }
}

// This macro generates the From<u8> and Into<u8> implementations for the
// register type
macro_rules! from_register_u8 {
    ($typ:ty) => {
        impl From<u8> for $typ {
            fn from(v: u8) -> Self {
                paste::item!([< $typ >](v))
            }
        }
        impl From<$typ> for u8 {
            fn from(v: $typ) -> Self {
                v.0
            }
        }
    };
}

bitfield! {
    /// Seconds register (0-59) with BCD encoding.
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Seconds(u8);
    impl Debug;
    /// Tens place of seconds (0-5)
    pub ten_seconds, set_ten_seconds: 6, 4;
    /// Ones place of seconds (0-9)
    pub seconds, set_seconds: 3, 0;
}
from_register_u8!(Seconds);

bitfield! {
    /// Minutes register (0-59) with BCD encoding.
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Minutes(u8);
    impl Debug;
    /// Tens place of minutes (0-5)
    pub ten_minutes, set_ten_minutes: 6, 4;
    /// Ones place of minutes (0-9)
    pub minutes, set_minutes: 3, 0;
}
from_register_u8!(Minutes);

bitfield! {
    /// Hours register with format selection and BCD encoding.
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Hours(u8);
    impl Debug;
    /// Time representation format (12/24 hour)
    pub from into TimeRepresentation, time_representation, set_time_representation: 6, 6;
    /// PM flag (12-hour) or 20-hour bit (24-hour)
    pub pm_or_twenty_hours, set_pm_or_twenty_hours: 5, 5;
    /// Tens place of hours
    pub ten_hours, set_ten_hours: 4, 4;
    /// Ones place of hours
    pub hours, set_hours: 3, 0;
}
from_register_u8!(Hours);

bitfield! {
    /// Day of week register (1-7).
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Day(u8);
    impl Debug;
    /// Day of week (1-7)
    pub day, set_day: 2, 0;
}
from_register_u8!(Day);

bitfield! {
    /// Date register (1-31) with BCD encoding.
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Date(u8);
    impl Debug;
    /// Tens place of date (0-3)
    pub ten_date, set_ten_date: 5, 4;
    /// Ones place of date (0-9)
    pub date, set_date: 3, 0;
}
from_register_u8!(Date);

bitfield! {
    /// Month register (1-12) with century flag and BCD encoding.
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Month(u8);
    impl Debug;
    /// Century flag (1 = year 2000+)
    pub century, set_century: 7;
    /// Tens place of month (0-1)
    pub ten_month, set_ten_month: 4, 4;
    /// Ones place of month (0-9)
    pub month, set_month: 3, 0;
}
from_register_u8!(Month);

bitfield! {
    /// Year register (0-99) with BCD encoding.
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Year(u8);
    impl Debug;
    /// Tens place of year (0-9)
    pub ten_year, set_ten_year: 7, 4;
    /// Ones place of year (0-9)
    pub year, set_year: 3, 0;
}
from_register_u8!(Year);

bitfield! {
    /// Control register for device configuration.
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Control(u8);
    impl Debug;
    /// Oscillator enable/disable control
    pub from into Ocillator, oscillator_enable, set_oscillator_enable: 7, 7;
    /// Enable square wave output on battery power
    pub battery_backed_square_wave, set_battery_backed_square_wave: 6;
    /// Force temperature conversion
    pub convert_temperature, set_convert_temperature: 5;
    /// Square wave output frequency selection
    pub from into SquareWaveFrequency, square_wave_frequency, set_square_wave_frequency: 4, 3;
    /// INT/SQW pin function control
    pub from into InterruptControl, interrupt_control, set_interrupt_control: 2, 2;
    /// Enable alarm 2 interrupt
    pub alarm2_interrupt_enable, set_alarm2_interrupt_enable: 1;
    /// Enable alarm 1 interrupt
    pub alarm1_interrupt_enable, set_alarm1_interrupt_enable: 0;
}
from_register_u8!(Control);

#[cfg(feature = "defmt")]
impl defmt::Format for Control {
    fn format(&self, f: defmt::Formatter) {
        match self.oscillator_enable() {
            Ocillator::Enabled => defmt::write!(f, "Oscillator enabled"),
            Ocillator::Disabled => defmt::write!(f, "Oscillator disabled"),
        }
        if self.battery_backed_square_wave() {
            defmt::write!(f, ", Battery backed square wave enabled");
        }
        if self.convert_temperature() {
            defmt::write!(f, ", Temperature conversion enabled");
        }
        match self.square_wave_frequency() {
            SquareWaveFrequency::Hz1 => defmt::write!(f, ", 1 Hz square wave"),
            SquareWaveFrequency::Hz1024 => defmt::write!(f, ", 1024 Hz square wave"),
            SquareWaveFrequency::Hz4096 => defmt::write!(f, ", 4096 Hz square wave"),
            SquareWaveFrequency::Hz8192 => defmt::write!(f, ", 8192 Hz square wave"),
        }
        match self.interrupt_control() {
            InterruptControl::SquareWave => defmt::write!(f, ", Square wave output"),
            InterruptControl::Interrupt => defmt::write!(f, ", Interrupt output"),
        }
        if self.alarm2_interrupt_enable() {
            defmt::write!(f, ", Alarm 2 interrupt enabled");
        }
        if self.alarm1_interrupt_enable() {
            defmt::write!(f, ", Alarm 1 interrupt enabled");
        }
    }
}

bitfield! {
    /// Status register for device state and flags.
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Status(u8);
    impl Debug;
    /// Oscillator stop flag
    pub oscillator_stop_flag, set_oscillator_stop_flag: 7;
    /// Enable 32kHz output
    pub enable_32khz_output, set_enable_32khz_output: 3;
    /// Device busy flag
    pub busy, set_busy: 2;
    /// Alarm 2 triggered flag
    pub alarm2_flag, set_alarm2_flag: 1;
    /// Alarm 1 triggered flag
    pub alarm1_flag, set_alarm1_flag: 0;
}
from_register_u8!(Status);

bitfield! {
    /// Aging offset register for oscillator adjustment.
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct AgingOffset(u8);
    impl Debug;
    /// Aging offset value (-128 to +127)
    pub i8, aging_offset, set_aging_offset: 7, 0;
}
from_register_u8!(AgingOffset);

bitfield! {
    /// Temperature register (integer part).
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Temperature(u8);
    impl Debug;
    /// Temperature value (-128 to +127)
    pub i8, temperature, set_temperature: 7, 0;
}
from_register_u8!(Temperature);

bitfield! {
    /// Temperature fraction register (decimal part).
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct TemperatureFraction(u8);
    impl Debug;
    /// Temperature fraction value (0.00 to 0.99)
    pub temperature_fraction, set_temperature_fraction: 7, 0;
}
from_register_u8!(TemperatureFraction);

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
}

#[cfg(feature = "async")]
#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec;

    use super::*;
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

    #[tokio::test]
    async fn test_async_alarm_registers() {
        let mock = setup_mock(&[
            // Test alarm1 registers
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm1Seconds as u8], vec![0x30]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm1Minutes as u8], vec![0x45]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm1Hours as u8], vec![0x12]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm1DayDate as u8], vec![0x15]),
            // Test alarm2 registers
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm2Minutes as u8], vec![0x30]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm2Hours as u8], vec![0x08]),
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::Alarm2DayDate as u8], vec![0x20]),
            // Test setting alarm registers
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Seconds as u8, 0x00]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Minutes as u8, 0x15]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1Hours as u8, 0x09]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm1DayDate as u8, 0x10]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2Minutes as u8, 0x45]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2Hours as u8, 0x14]),
            I2cTrans::write(DEVICE_ADDRESS, vec![RegAddr::Alarm2DayDate as u8, 0x25]),
        ])
        .await;

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
        dev.set_alarm1_second(Seconds(0x00)).await.unwrap();
        dev.set_alarm1_minute(Minutes(0x15)).await.unwrap();
        dev.set_alarm1_hour(Hours(0x09)).await.unwrap();
        dev.set_alarm1_day_date(Date(0x10)).await.unwrap();
        dev.set_alarm2_minute(Minutes(0x45)).await.unwrap();
        dev.set_alarm2_hour(Hours(0x14)).await.unwrap();
        dev.set_alarm2_day_date(Date(0x25)).await.unwrap();

        dev.i2c.done();
    }

    #[tokio::test]
    async fn test_async_status_register_flags() {
        let mock = setup_mock(&[
            // Test various status flag combinations
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::ControlStatus as u8], vec![0x00]), // All flags clear
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::ControlStatus as u8], vec![0x88]), // OSF and EN32kHz set
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::ControlStatus as u8], vec![0x07]), // BSY, A2F, A1F set
            I2cTrans::write_read(DEVICE_ADDRESS, vec![RegAddr::ControlStatus as u8], vec![0x8F]), // All flags set
        ])
        .await;

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

    #[tokio::test]
    async fn test_async_individual_registers() {
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
        ])
        .await;

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

    #[tokio::test]
    async fn test_async_twelve_hour_mode() {
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
        ])
        .await;

        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);
        dev.configure(&config).await.unwrap();
        assert_eq!(dev.time_representation, TimeRepresentation::TwelveHour);
        dev.i2c.done();
    }

    #[test]
    fn test_register_from_u8_conversions() {
        // Test TimeRepresentation conversions
        assert_eq!(TimeRepresentation::from(0), TimeRepresentation::TwentyFourHour);
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
}

#[cfg(not(feature = "async"))]
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

    #[test]
    fn test_read_control() {
        // Control register value: oscillator enabled, 1Hz square wave (bits 4,3 = 0b00)
        let expected = 0b0000_0000; // Hz1 frequency (0b00 in bits 4,3)
        let mock = setup_mock(&[I2cTrans::write_read(
            DEVICE_ADDRESS,
            vec![RegAddr::Control as u8],
            vec![expected],
        )]);
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        let control = dev.control().unwrap();
        assert_eq!(control.oscillator_enable(), Ocillator::Enabled);
        assert_eq!(control.square_wave_frequency(), SquareWaveFrequency::Hz1);
        assert_eq!(control.interrupt_control(), InterruptControl::SquareWave);
        dev.i2c.done();
    }

    #[test]
    fn test_write_control() {
        let mut control = Control::default();
        control.set_oscillator_enable(Ocillator::Enabled);
        control.set_square_wave_frequency(SquareWaveFrequency::Hz1024);

        let mock = setup_mock(&[I2cTrans::write(
            DEVICE_ADDRESS,
            vec![RegAddr::Control as u8, control.0],
        )]);
        let mut dev = DS3231::new(mock, DEVICE_ADDRESS);

        dev.set_control(control).unwrap();
        dev.i2c.done();
    }

    #[test]
    fn test_configure() {
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
        dev.configure(&config).unwrap();
        dev.i2c.done();
    }

    #[test]
    fn test_read_datetime() {
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

        let dt = dev.datetime().unwrap();
        assert_eq!(dt.hour(), 15);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 0);
        assert_eq!(dt.day(), 14);
        assert_eq!(dt.month(), 3);
        assert_eq!(dt.year(), 2024);
        dev.i2c.done();
    }

    #[test]
    fn test_set_datetime() {
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

        dev.set_datetime(&dt).unwrap();
        dev.i2c.done();
    }

    #[test]
    fn test_register_operations() {
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
        let seconds = dev.second().unwrap();
        assert_eq!(seconds.seconds(), 5);
        assert_eq!(seconds.ten_seconds(), 4);
        dev.set_second(Seconds(0x30)).unwrap();

        // Test minutes
        let minutes = dev.minute().unwrap();
        assert_eq!(minutes.minutes(), 0);
        assert_eq!(minutes.ten_minutes(), 3);
        dev.set_minute(Minutes(0x45)).unwrap();

        // Test status
        let status = dev.status().unwrap();
        assert!(status.oscillator_stop_flag());

        dev.i2c.done();
    }

    #[test]
    fn test_read_temperature() {
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

        let temp = dev.temperature().unwrap();
        let frac = dev.temperature_fraction().unwrap();
        assert_eq!(temp.temperature(), 25);
        assert_eq!(frac.temperature_fraction(), 0x60);
        dev.i2c.done();
    }

    #[test]
    fn test_enum_conversions() {
        // Test TimeRepresentation conversions
        assert_eq!(TimeRepresentation::from(0), TimeRepresentation::TwentyFourHour);
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
}
