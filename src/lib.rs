#![no_std]

use bilge::prelude::*;
use bilge::BitsError;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use embedded_hal_async::i2c::I2c;
use log::debug;

#[bitsize(1)]
#[derive(FromBits, Copy, Clone, Debug, PartialEq)]
pub enum TimeRepresentation {
    TwentyFourHour = 0,
    TwelveHour = 1,
}

#[bitsize(1)]
#[derive(FromBits, Copy, Clone, Debug, PartialEq)]
pub enum Ocillator {
    Enabled = 0,
    Disabled = 1,
}

#[bitsize(1)]
#[derive(FromBits, Copy, Clone, Debug, PartialEq)]
pub enum InterruptControl {
    SquareWave = 0,
    Interrupt = 1,
}

#[bitsize(2)]
#[derive(FromBits, Copy, Clone, Debug, PartialEq)]
pub enum SquareWaveFrequency {
    Hz1 = 0b00,
    Hz1024 = 0b01,
    Hz4096 = 0b10,
    Hz8192 = 0b11,
}

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

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct Seconds {
    pub seconds: u4,
    pub ten_seconds: u3,
    reserved: u1,
}

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct Minutes {
    pub minutes: u4,
    pub ten_minutes: u3,
    reserved: u1,
}

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct Hours {
    pub hours: u4,
    pub ten_hours: u1,
    pub pm_or_twenty_hours: u1,
    pub time_representation: TimeRepresentation,
    reserved: u1,
}

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct Day {
    pub day: u3,
    reserved: u5,
}

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct Date {
    pub date: u4,
    pub ten_date: u2,
    reserved: u2,
}

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct Month {
    pub month: u4,
    pub ten_month: u1,
    pub century: u1,
    reserved: u2,
}

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct Year {
    pub year: u4,
    pub ten_year: u4,
}

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct Control {
    pub alarm1_interrupt_enable: bool,
    pub alarm2_interrupt_enable: bool,
    pub interrupt_control: InterruptControl,
    pub square_wave_frequency: SquareWaveFrequency,
    pub convert_temperature: bool,
    pub battery_backed_square_wave: bool,
    pub oscillator_enable: Ocillator,
}

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct Status {
    pub alarm1_flag: bool,
    pub alarm2_flag: bool,
    pub busy: bool,
    pub enable_32khz_output: bool,
    reserved: u3,
    pub oscillator_stop_flag: bool,
}

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct Temperature {
    pub temperature: u8,
}

#[bitsize(8)]
#[derive(TryFromBits, DebugBits, Copy, Clone, PartialEq)]
pub struct TemperatureFraction {
    reserved: u6,
    pub temperature_fraction: u2,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct DS3231DateTime {
    seconds: Seconds,
    minutes: Minutes,
    hours: Hours,
    day: Day,
    date: Date,
    month: Month,
    year: Year,
}

macro_rules! set_and_get_register {
    ($(($name:ident, $regaddr:expr, $typ:ty)),+) => {
        $(
            paste::item!{
                pub async fn [< set_ $name >](&mut self, value: $typ) -> Result<(), DS2131Error<I2C::Error>> {
                    self.i2c.write(
                        self.address,
                        &[$regaddr as u8, value.value],
                        )
                        .await?;
                    Ok(())
                }
            }

            pub async fn $name(&mut self) -> Result<$typ, DS2131Error<I2C::Error>> {
                let mut data = [0];
                self.i2c
                    .write_read(self.address, &[$regaddr as u8], &mut data)
                    .await?;
                match paste::item!([< $typ >]::try_from)(data[0]) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(DS2131Error::BitsError(e)),
                }
            }
        )+
    }
}

#[derive(Debug)]
pub enum DS2131Error<I2CE> {
    I2c(I2CE),
    BitsError(BitsError),
    SecondsBitsError(BitsError),
    MinutesBitsError(BitsError),
    HoursBitsError(BitsError),
    DayBitsError(BitsError),
    DateBitsError(BitsError),
    MonthBitsError(BitsError),
    YearBitsError(BitsError),
}

impl<I2CE> From<I2CE> for DS2131Error<I2CE> {
    fn from(e: I2CE) -> Self {
        DS2131Error::I2c(e)
    }
}

pub struct DS3231<I2C: I2c> {
    i2c: I2C,
    address: u8,
}

#[allow(unused)]
impl<I2C: I2c> DS3231<I2C> {
    pub fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    pub async fn configure(&mut self, config: &Config) -> Result<(), DS2131Error<I2C::Error>> {
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

        Ok(())
    }

    async fn read_raw_datetime(&mut self) -> Result<DS3231DateTime, DS2131Error<I2C::Error>> {
        let mut data = [0; 7];
        self.i2c
            .write_read(self.address, &[RegAddr::Seconds as u8], &mut data)
            .await?;
        Ok(DS3231DateTime {
            seconds: Seconds::try_from(data[0]).map_err(DS2131Error::SecondsBitsError)?,
            minutes: Minutes::try_from(data[1]).map_err(DS2131Error::MinutesBitsError)?,
            hours: Hours::try_from(data[2]).map_err(DS2131Error::HoursBitsError)?,
            day: Day::try_from(data[3]).map_err(DS2131Error::DayBitsError)?,
            date: Date::try_from(data[4]).map_err(DS2131Error::DateBitsError)?,
            month: Month::try_from(data[5]).map_err(DS2131Error::MonthBitsError)?,
            year: Year::try_from(data[6]).map_err(DS2131Error::YearBitsError)?,
        })
    }

    async fn write_raw_datetime(
        &mut self,
        datetime: &DS3231DateTime,
    ) -> Result<(), DS2131Error<I2C::Error>> {
        self.i2c
            .write(
                self.address,
                &[
                    RegAddr::Seconds as u8,
                    datetime.seconds.value,
                    datetime.minutes.value,
                    datetime.hours.value,
                    datetime.day.value,
                    datetime.date.value,
                    datetime.month.value,
                    datetime.year.value,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn datetime(&mut self) -> Result<NaiveDateTime, DS2131Error<I2C::Error>> {
        let raw = self.read_raw_datetime().await?;
        let seconds = 10 * u32::from(raw.seconds.ten_seconds()) + u32::from(raw.seconds.seconds());
        let minutes = 10 * u32::from(raw.minutes.ten_minutes()) + u32::from(raw.minutes.minutes());
        let hours = 10 * u32::from(raw.hours.ten_hours()) + u32::from(raw.hours.hours());
        let hours = match raw.hours.time_representation() {
            TimeRepresentation::TwentyFourHour => {
                hours + 20 * u32::from(raw.hours.pm_or_twenty_hours())
            }
            TimeRepresentation::TwelveHour => {
                hours + 12 * u32::from(raw.hours.pm_or_twenty_hours())
            }
        };
        debug!(
            "raw_hour={:08b} h={} m={} s={}",
            raw.hours.value, hours, minutes, seconds
        );
        Ok(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(
                2000 + (10 * u32::from(raw.year.ten_year()) + u32::from(raw.year.year())) as i32,
                10 * u32::from(raw.month.ten_month()) + u32::from(raw.month.month()),
                (10 * u32::from(raw.date.ten_date()) + u32::from(raw.date.date())),
            )
            .expect("Invalid date"),
            NaiveTime::from_hms_opt(hours, minutes, seconds).expect("Invalid time"),
        ))
    }

    pub async fn set_datetime(
        &mut self,
        datetime: &NaiveDateTime,
    ) -> Result<(), DS2131Error<I2C::Error>> {
        todo!()
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
