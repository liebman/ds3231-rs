#![no_std]
mod datetime;

use bitfield::bitfield;
use chrono::DateTime;
use chrono::Utc;
use datetime::DS3231DateTimeError;
#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;
use log::debug;

use crate::datetime::DS3231DateTime;

pub struct Config {
    pub time_representation: TimeRepresentation,
    pub square_wave_frequency: SquareWaveFrequency,
    pub interrupt_control: InterruptControl,
    pub battery_backed_square_wave: bool,
    pub oscillator_enable: Ocillator,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RegAddr {
    Seconds = 0x00,
    Minutes = 0x01,
    Hours = 0x02,
    Day = 0x03,
    Date = 0x04,
    Month = 0x05,
    Year = 0x06,
    Alarm1Seconds = 0x07,
    Alarm1Minutes = 0x08,
    Alarm1Hours = 0x09,
    Alarm1DayDate = 0x0A,
    Alarm2Minutes = 0x0B,
    Alarm2Hours = 0x0C,
    Alarm2DayDate = 0x0D,
    Control = 0x0E,
    ControlStatus = 0x0F,
    AgingOffset = 0x10,
    MSBTemp = 0x11,
    LSBTemp = 0x12,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeRepresentation {
    TwentyFourHour = 0,
    TwelveHour = 1,
}
impl From<u8> for TimeRepresentation {
    fn from(v: u8) -> Self {
        match v {
            0 => TimeRepresentation::TwentyFourHour,
            1 => TimeRepresentation::TwelveHour,
            _ => panic!("Invalid value for TimeRepresentation: {}", v),
        }
    }
}
impl From<TimeRepresentation> for u8 {
    fn from(v: TimeRepresentation) -> Self {
        v as u8
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Ocillator {
    Enabled = 0,
    Disabled = 1,
}
impl From<u8> for Ocillator {
    fn from(v: u8) -> Self {
        match v {
            0 => Ocillator::Enabled,
            1 => Ocillator::Disabled,
            _ => panic!("Invalid value for Ocillator: {}", v),
        }
    }
}
impl From<Ocillator> for u8 {
    fn from(v: Ocillator) -> Self {
        v as u8
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InterruptControl {
    SquareWave = 0,
    Interrupt = 1,
}
impl From<u8> for InterruptControl {
    fn from(v: u8) -> Self {
        match v {
            0 => InterruptControl::SquareWave,
            1 => InterruptControl::Interrupt,
            _ => panic!("Invalid value for InterruptControl: {}", v),
        }
    }
}
impl From<InterruptControl> for u8 {
    fn from(v: InterruptControl) -> Self {
        v as u8
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SquareWaveFrequency {
    Hz1 = 0b00,
    Hz1024 = 0b01,
    Hz4096 = 0b10,
    Hz8192 = 0b11,
}
impl From<u8> for SquareWaveFrequency {
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
    fn from(v: SquareWaveFrequency) -> Self {
        v as u8
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
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Seconds(u8);
    impl Debug;
    pub ten_seconds, set_ten_seconds: 6, 4;
    pub seconds, set_seconds: 3, 0;
}
from_register_u8!(Seconds);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Minutes(u8);
    impl Debug;
    pub ten_minutes, set_ten_minutes: 6, 4;
    pub minutes, set_minutes: 3, 0;
}
from_register_u8!(Minutes);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Hours(u8);
    impl Debug;
    pub from into TimeRepresentation, time_representation, set_time_representation: 6, 6;
    pub pm_or_twenty_hours, set_pm_or_twenty_hours: 5, 5;
    pub ten_hours, set_ten_hours: 4, 4;
    pub hours, set_hours: 3, 0;
}
from_register_u8!(Hours);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Day(u8);
    impl Debug;
    pub day, set_day: 2, 0;
}
from_register_u8!(Day);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Date(u8);
    impl Debug;
    pub ten_date, set_ten_date: 5, 4;
    pub date, set_date: 3, 0;
}
from_register_u8!(Date);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Month(u8);
    impl Debug;
    pub century, set_century: 7;
    pub ten_month, set_ten_month: 4, 4;
    pub month, set_month: 3, 0;
}
from_register_u8!(Month);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Year(u8);
    impl Debug;
    pub ten_year, set_ten_year: 7, 4;
    pub year, set_year: 3, 0;
}
from_register_u8!(Year);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Control(u8);
    impl Debug;
    pub from into Ocillator, oscillator_enable, set_oscillator_enable: 7, 7;
    pub battery_backed_square_wave, set_battery_backed_square_wave: 6;
    pub convert_temperature, set_convert_temperature: 5;
    pub from into SquareWaveFrequency, square_wave_frequency, set_square_wave_frequency: 4, 3;
    pub from into InterruptControl, interrupt_control, set_interrupt_control: 2, 2;
    pub alarm2_interrupt_enable, set_alarm2_interrupt_enable: 1;
    pub alarm1_interrupt_enable, set_alarm1_interrupt_enable: 0;
}
from_register_u8!(Control);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Status(u8);
    impl Debug;
    pub oscillator_stop_flag, set_oscillator_stop_flag: 7;
    pub enable_32khz_output, set_enable_32khz_output: 3;
    pub busy, set_busy: 2;
    pub alarm2_flag, set_alarm2_flag: 1;
    pub alarm1_flag, set_alarm1_flag: 0;
}
from_register_u8!(Status);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct AgingOffset(u8);
    impl Debug;
    pub i8, aging_offset, set_aging_offset: 7, 0;
}
from_register_u8!(AgingOffset);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Temperature(u8);
    impl Debug;
    pub i8, temperature, set_temperature: 7, 0;
}
from_register_u8!(Temperature);

bitfield! {
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct TemperatureFraction(u8);
    impl Debug;
    pub temperature_fraction, set_temperature_fraction: 7, 0;
}
from_register_u8!(TemperatureFraction);

macro_rules! set_and_get_register {
    ($(($name:ident, $regaddr:expr, $typ:ty)),+) => {
        $(
            paste::item!{
                #[cfg(not(feature = "async"))]
                pub fn [< set_ $name >](&mut self, value: $typ) -> Result<(), DS3231Error<I2C::Error>> {
                    self.i2c.write(
                        self.address,
                        &[$regaddr as u8, value.into()],
                        )?;
                    Ok(())
                }
                #[cfg(feature = "async")]
                pub async fn [< set_ $name >](&mut self, value: $typ) -> Result<(), DS3231Error<I2C::Error>> {
                    self.i2c.write(
                        self.address,
                        &[$regaddr as u8, value.into()],
                        )
                        .await?;
                    Ok(())
                }
            }

            #[cfg(not(feature = "async"))]
            pub fn $name(&mut self) -> Result<$typ, DS3231Error<I2C::Error>> {
                let mut data = [0];
                self.i2c
                    .write_read(self.address, &[$regaddr as u8], &mut data)?;
                Ok(paste::item!([<$typ>])(data[0]))
            }
            #[cfg(feature = "async")]
            pub async fn $name(&mut self) -> Result<$typ, DS3231Error<I2C::Error>> {
                let mut data = [0];
                self.i2c
                    .write_read(self.address, &[$regaddr as u8], &mut data)
                    .await?;
                Ok(paste::item!([<$typ>])(data[0]))
            }
        )+
    }
}

#[derive(Debug)]
pub enum DS3231Error<I2CE> {
    I2c(I2CE),
    DateTime(DS3231DateTimeError),
}

impl<I2CE> From<I2CE> for DS3231Error<I2CE> {
    fn from(e: I2CE) -> Self {
        DS3231Error::I2c(e)
    }
}

// impl<I2CE> From<DS3231DateTimeError> for DS3231Error<I2CE>
// {
//     fn from(e: DS3231DateTimeError) -> Self {
//         DS3231Error::DateTime(e)
//     }
// }

pub struct DS3231<I2C: I2c> {
    i2c: I2C,
    address: u8,
    time_representation: TimeRepresentation,
}

#[allow(unused)]
impl<I2C: I2c> DS3231<I2C> {
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            time_representation: TimeRepresentation::TwentyFourHour,
        }
    }

    #[cfg(not(feature = "async"))]
    pub fn configure(&mut self, config: &Config) -> Result<(), DS3231Error<I2C::Error>> {
        let mut control = self.control()?;
        control.set_oscillator_enable(config.oscillator_enable);
        control.set_battery_backed_square_wave(config.battery_backed_square_wave);
        control.set_square_wave_frequency(config.square_wave_frequency);
        control.set_interrupt_control(config.interrupt_control);
        debug!("control: {:?}", control);
        self.set_control(control)?;

        let mut hours = self.hour()?;
        hours.set_time_representation(config.time_representation);
        self.set_hour(hours)?;
        self.time_representation = config.time_representation;
        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn configure(&mut self, config: &Config) -> Result<(), DS3231Error<I2C::Error>> {
        let mut control = self.control().await?;
        control.set_oscillator_enable(config.oscillator_enable);
        control.set_battery_backed_square_wave(config.battery_backed_square_wave);
        control.set_square_wave_frequency(config.square_wave_frequency);
        control.set_interrupt_control(config.interrupt_control);
        debug!("control: {:?}", control);
        self.set_control(control).await?;

        let mut hours = self.hour().await?;
        hours.set_time_representation(config.time_representation);
        self.set_hour(hours).await?;
        self.time_representation = config.time_representation;
        Ok(())
    }

    #[cfg(not(feature = "async"))]
    fn read_raw_datetime(&mut self) -> Result<DS3231DateTime, DS3231Error<I2C::Error>> {
        let mut data = [0; 7];
        self.i2c
            .write_read(self.address, &[RegAddr::Seconds as u8], &mut data)?;
        Ok(data.into())
    }

    #[cfg(feature = "async")]
    async fn read_raw_datetime(&mut self) -> Result<DS3231DateTime, DS3231Error<I2C::Error>> {
        let mut data = [0; 7];
        self.i2c
            .write_read(self.address, &[RegAddr::Seconds as u8], &mut data)
            .await?;
        Ok(data.into())
    }

    #[cfg(not(feature = "async"))]
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
        );
        Ok(())
    }

    #[cfg(feature = "async")]
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

    #[cfg(not(feature = "async"))]
    pub fn datetime(&mut self) -> Result<DateTime<Utc>, DS3231Error<I2C::Error>> {
        let raw = self.read_raw_datetime()?;
        raw.into_datetime().map_err(DS3231Error::DateTime)
    }

    #[cfg(feature = "async")]
    pub async fn datetime(&mut self) -> Result<DateTime<Utc>, DS3231Error<I2C::Error>> {
        let raw = self.read_raw_datetime().await?;
        raw.into_datetime().map_err(DS3231Error::DateTime)
    }

    #[cfg(not(feature = "async"))]
    pub async fn set_datetime(
        &mut self,
        datetime: &DateTime<Utc>,
    ) -> Result<(), DS3231Error<I2C::Error>> {
        let raw = DS3231DateTime::from_datetime(datetime, self.time_representation)
            .map_err(DS3231Error::DateTime)?;
        self.write_raw_datetime(&raw)?;
        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn set_datetime(
        &mut self,
        datetime: &DateTime<Utc>,
    ) -> Result<(), DS3231Error<I2C::Error>> {
        let raw = DS3231DateTime::from_datetime(datetime, self.time_representation)
            .map_err(DS3231Error::DateTime)?;
        self.write_raw_datetime(&raw).await?;
        Ok(())
    }

    set_and_get_register!(
        (second, RegAddr::Seconds, Seconds),
        (minute, RegAddr::Minutes, Minutes),
        (hour, RegAddr::Hours, Hours),
        (date, RegAddr::Date, Date),
        (month, RegAddr::Month, Month),
        (year, RegAddr::Year, Year),
        (control, RegAddr::Control, Control),
        (status, RegAddr::ControlStatus, Status)
    );
}
