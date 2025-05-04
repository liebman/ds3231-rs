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
//! - DateTime validation errors
//!
//! ## Safety
//!
//! This driver uses no `unsafe` code and ensures type safety through:
//! - Strong typing for all register operations
//! - Validation of all datetime values
//! - Proper error propagation

#![no_std]
#![warn(missing_docs)]
// MUST be the first module
mod fmt;

#[cfg(feature = "async")]
pub mod asynch;
mod datetime;

use bitfield::bitfield;
use chrono::NaiveDateTime;
use datetime::DS3231DateTimeError;
use embedded_hal::i2c::I2c;
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
    /// Creates a TimeRepresentation from a raw register value.
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
    /// Converts a TimeRepresentation to its raw register value.
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
    /// Creates an InterruptControl from a raw register value.
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
    /// Converts an InterruptControl to its raw register value.
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
    /// Creates a SquareWaveFrequency from a raw register value.
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
    /// Converts a SquareWaveFrequency to its raw register value.
    fn from(v: SquareWaveFrequency) -> Self {
        v as u8
    }
}

/// Error type for DS3231 operations.
#[derive(Debug)]
pub enum DS3231Error<I2CE> {
    /// I2C bus error
    I2c(I2CE),
    /// DateTime validation or conversion error
    DateTime(DS3231DateTimeError),
}

impl<I2CE> From<I2CE> for DS3231Error<I2CE> {
    /// Creates a DS3231Error from an I2C error.
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
/// For async operations, see the [`asynch`] module.
pub struct DS3231<I2C: I2c> {
    i2c: I2C,
    address: u8,
    time_representation: TimeRepresentation,
}

impl<I2C: I2c> DS3231<I2C> {
    /// Creates a new DS3231 driver instance.
    ///
    /// # Arguments
    /// * `i2c` - The I2C bus implementation
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
    pub fn configure(&mut self, config: &Config) -> Result<(), DS3231Error<I2C::Error>> {
        let mut control = self.control()?;
        control.set_oscillator_enable(config.oscillator_enable);
        control.set_battery_backed_square_wave(config.battery_backed_square_wave);
        control.set_square_wave_frequency(config.square_wave_frequency);
        control.set_interrupt_control(config.interrupt_control);
        #[cfg(any(feature = "log", feature = "defmt"))]
        debug!("control: {:?}", control);
        self.set_control(control)?;

        let mut hours = self.hour()?;
        hours.set_time_representation(config.time_representation);
        self.set_hour(hours)?;
        self.time_representation = config.time_representation;
        Ok(())
    }

    /// Reads the raw datetime registers from the device.
    ///
    /// # Returns
    /// * `Ok(DS3231DateTime)` - The raw datetime values on success
    /// * `Err(DS3231Error)` on error
    fn read_raw_datetime(&mut self) -> Result<DS3231DateTime, DS3231Error<I2C::Error>> {
        let mut data = [0; 7];
        self.i2c
            .write_read(self.address, &[RegAddr::Seconds as u8], &mut data)?;
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
    fn write_raw_datetime(
        &mut self,
        datetime: &DS3231DateTime,
    ) -> Result<(), DS3231Error<I2C::Error>> {
        let data: [u8; 7] = datetime.into();
        self.i2c.write(
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
        )?;
        Ok(())
    }

    /// Gets the current date and time from the device.
    ///
    /// # Returns
    /// * `Ok(NaiveDateTime)` - The current date and time
    /// * `Err(DS3231Error)` on error
    pub fn datetime(&mut self) -> Result<NaiveDateTime, DS3231Error<I2C::Error>> {
        let raw = self.read_raw_datetime()?;
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
    pub fn set_datetime(
        &mut self,
        datetime: &NaiveDateTime,
    ) -> Result<(), DS3231Error<I2C::Error>> {
        let raw = DS3231DateTime::from_datetime(datetime, self.time_representation)
            .map_err(DS3231Error::DateTime)?;
        self.write_raw_datetime(&raw)?;
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
                    pub fn $name(&mut self) -> Result<$typ, DS3231Error<I2C::Error>> {
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
                    pub fn [<set_ $name>](&mut self, value: $typ) -> Result<(), DS3231Error<I2C::Error>> {
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
}
