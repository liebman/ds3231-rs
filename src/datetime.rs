//! `DateTime` conversion and register utilities for the DS3231 RTC.
//!
//! This module provides the internal representation and conversion logic for the DS3231's date and time registers.
//! It enables safe, validated conversion between the DS3231's BCD-encoded registers and chrono's `NaiveDateTime`.
//!
//! # Features
//!
//! - Conversion to/from chrono `NaiveDateTime`
//! - Error handling for invalid or out-of-range values
//!
//! # Register Model
//!
//! The DS3231 stores date and time in 7 consecutive registers:
//! - Seconds, Minutes, Hours, Day, Date, Month, Year
//!
//! # Error Handling
//!
//! Conversion errors are reported via [`DS3231DateTimeError`].

use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};

use crate::{Date, Day, Hours, Minutes, Month, Seconds, TimeRepresentation, Year};

/// Internal representation of the DS3231 RTC date and time.
///
/// This struct models the 7 date/time registers of the DS3231, using strongly-typed bitfield wrappers for each field.
/// It is used for register-level I/O and conversion to/from chrono's `NaiveDateTime`.
///
/// Values are always validated and encoded/decoded as BCD.
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct DS3231DateTime {
    seconds: Seconds,
    minutes: Minutes,
    hours: Hours,
    day: Day,
    date: Date,
    month: Month,
    year: Year,
}

impl DS3231DateTime {
    /// Helper function to convert a number to BCD format with validation
    fn make_bcd(value: u32, max_value: u32) -> Result<(u8, u8), DS3231DateTimeError> {
        if value > max_value {
            return Err(DS3231DateTimeError::InvalidDateTime);
        }
        let ones = u8::try_from(value % 10).map_err(|_| DS3231DateTimeError::InvalidDateTime)?;
        let tens = u8::try_from(value / 10).map_err(|_| DS3231DateTimeError::InvalidDateTime)?;
        Ok((ones, tens))
    }

    fn convert_seconds(seconds: u32) -> Result<Seconds, DS3231DateTimeError> {
        let (ones, tens) = Self::make_bcd(seconds, 59)?;
        let mut value = Seconds::default();
        value.set_seconds(ones);
        value.set_ten_seconds(tens);
        Ok(value)
    }

    fn convert_minutes(minutes: u32) -> Result<Minutes, DS3231DateTimeError> {
        let (ones, tens) = Self::make_bcd(minutes, 59)?;
        let mut value = Minutes::default();
        value.set_minutes(ones);
        value.set_ten_minutes(tens);
        Ok(value)
    }

    fn convert_hours(
        hour: u32,
        time_representation: TimeRepresentation,
    ) -> Result<Hours, DS3231DateTimeError> {
        if hour > 23 {
            return Err(DS3231DateTimeError::InvalidDateTime);
        }
        let mut value = Hours::default();
        value.set_time_representation(time_representation);

        match time_representation {
            TimeRepresentation::TwentyFourHour => {
                let ones =
                    u8::try_from(hour % 10).map_err(|_| DS3231DateTimeError::InvalidDateTime)?;
                let ten = u8::try_from((hour / 10) & 0x01)
                    .map_err(|_| DS3231DateTimeError::InvalidDateTime)?;
                let twenty = u8::try_from((hour / 10) >> 1 & 0x01)
                    .map_err(|_| DS3231DateTimeError::InvalidDateTime)?;
                value.set_hours(ones);
                value.set_ten_hours(ten);
                value.set_pm_or_twenty_hours(twenty);
            }
            TimeRepresentation::TwelveHour => {
                let (hour12, is_pm) = match hour {
                    0 => (12, false),             // 12 AM
                    1..=11 => (hour, false),      // 1-11 AM
                    12 => (12, true),             // 12 PM
                    13..=23 => (hour - 12, true), // 1-11 PM
                    _ => unreachable!(),          // Already checked h <= 23
                };
                let ones =
                    u8::try_from(hour12 % 10).map_err(|_| DS3231DateTimeError::InvalidDateTime)?;
                let tens =
                    u8::try_from(hour12 / 10).map_err(|_| DS3231DateTimeError::InvalidDateTime)?;
                value.set_hours(ones);
                value.set_ten_hours(tens);
                value.set_pm_or_twenty_hours(u8::from(is_pm));
            }
        }
        Ok(value)
    }

    fn convert_day(weekday: u32) -> Result<Day, DS3231DateTimeError> {
        if weekday > 6 {
            return Err(DS3231DateTimeError::InvalidDateTime);
        }
        let mut value = Day::default();
        value.set_day(u8::try_from(weekday).map_err(|_| DS3231DateTimeError::InvalidDateTime)?);
        Ok(value)
    }

    fn convert_date(date: u32) -> Result<Date, DS3231DateTimeError> {
        let (ones, tens) = Self::make_bcd(date, 31)?;
        let mut value = Date::default();
        value.set_date(ones);
        value.set_ten_date(tens);
        Ok(value)
    }

    fn convert_month(month: u32) -> Result<Month, DS3231DateTimeError> {
        let (ones, tens) = Self::make_bcd(month, 12)?;
        let mut value = Month::default();
        value.set_month(ones);
        value.set_ten_month(tens);
        Ok(value)
    }

    fn convert_year(year: i32) -> Result<(Year, bool), DS3231DateTimeError> {
        if year > 2199 {
            #[cfg(any(feature = "log", feature = "defmt"))]
            error!("Year {} is too late! must be before 2200", year);
            return Err(DS3231DateTimeError::YearNotBefore2200);
        }
        if year < 2000 {
            #[cfg(any(feature = "log", feature = "defmt"))]
            error!("Year {} is too early! must be greater than 1999", year);
            return Err(DS3231DateTimeError::YearNotAfter1999);
        }

        let mut year_offset =
            u8::try_from(year - 2000).map_err(|_| DS3231DateTimeError::InvalidDateTime)?;
        let century = if year_offset > 99 {
            year_offset = year_offset.wrapping_sub(100);
            true
        } else {
            false
        };

        let ones = year_offset % 10;
        let tens = year_offset / 10;

        let mut value = Year::default();
        value.set_year(ones);
        value.set_ten_year(tens);
        Ok((value, century))
    }

    pub(crate) fn from_datetime(
        datetime: &NaiveDateTime,
        time_representation: TimeRepresentation,
    ) -> Result<Self, DS3231DateTimeError> {
        let seconds = Self::convert_seconds(datetime.second())?;
        let minutes = Self::convert_minutes(datetime.minute())?;
        let hours = Self::convert_hours(datetime.hour(), time_representation)?;
        let day = Self::convert_day(datetime.weekday().num_days_from_sunday())?;
        let date = Self::convert_date(datetime.day())?;
        let mut month = Self::convert_month(datetime.month())?;
        let (year, century) = Self::convert_year(datetime.year())?;

        if century {
            month.set_century(true);
        }

        let raw = DS3231DateTime {
            seconds,
            minutes,
            hours,
            day,
            date,
            month,
            year,
        };

        #[cfg(any(feature = "log", feature = "defmt"))]
        debug!("raw={:?}", raw);

        Ok(raw)
    }

    pub(crate) fn into_datetime(self) -> Result<NaiveDateTime, DS3231DateTimeError> {
        let seconds: u32 =
            10 * u32::from(self.seconds.ten_seconds()) + u32::from(self.seconds.seconds());
        let minutes =
            10 * u32::from(self.minutes.ten_minutes()) + u32::from(self.minutes.minutes());
        let hours = 10 * u32::from(self.hours.ten_hours()) + u32::from(self.hours.hours());
        let hours = match self.hours.time_representation() {
            TimeRepresentation::TwentyFourHour => {
                hours + 20 * u32::from(self.hours.pm_or_twenty_hours())
            }
            TimeRepresentation::TwelveHour => {
                hours + 12 * u32::from(self.hours.pm_or_twenty_hours())
            }
        };
        #[cfg(any(feature = "log", feature = "defmt"))]
        debug!(
            "raw_hour={:?} h={} m={} s={}",
            self.hours, hours, minutes, seconds
        );

        let year_offset = 10 * u32::from(self.year.ten_year()) + u32::from(self.year.year());
        let year = 2000_i32
            + i32::try_from(year_offset).map_err(|_| DS3231DateTimeError::InvalidDateTime)?;
        let month = 10 * u32::from(self.month.ten_month()) + u32::from(self.month.month());
        let date = 10 * u32::from(self.date.ten_date()) + u32::from(self.date.date());

        // Validate the date components before creating NaiveDateTime
        NaiveDate::from_ymd_opt(year, month, date)
            .and_then(|d| d.and_hms_opt(hours, minutes, seconds))
            .ok_or(DS3231DateTimeError::InvalidDateTime)
    }
}

impl From<[u8; 7]> for DS3231DateTime {
    fn from(data: [u8; 7]) -> Self {
        DS3231DateTime {
            seconds: Seconds(data[0]),
            minutes: Minutes(data[1]),
            hours: Hours(data[2]),
            day: Day(data[3]),
            date: Date(data[4]),
            month: Month(data[5]),
            year: Year(data[6]),
        }
    }
}

impl From<&DS3231DateTime> for [u8; 7] {
    fn from(dt: &DS3231DateTime) -> [u8; 7] {
        [
            dt.seconds.0,
            dt.minutes.0,
            dt.hours.0,
            dt.day.0,
            dt.date.0,
            dt.month.0,
            dt.year.0,
        ]
    }
}

#[derive(Debug)]
/// Errors that can occur during DS3231 date/time conversion or validation.
pub enum DS3231DateTimeError {
    /// The provided or decoded date/time is invalid (e.g., out of range, not representable)
    InvalidDateTime,
    /// The year is not before 2200 (DS3231 only supports years < 2200)
    YearNotBefore2200,
    /// The year is not after 1999 (DS3231 only supports years >= 2000)
    YearNotAfter1999,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_make_bcd_valid() {
        // Test valid cases
        assert_eq!(DS3231DateTime::make_bcd(0, 59).unwrap(), (0, 0));
        assert_eq!(DS3231DateTime::make_bcd(9, 59).unwrap(), (9, 0));
        assert_eq!(DS3231DateTime::make_bcd(10, 59).unwrap(), (0, 1));
        assert_eq!(DS3231DateTime::make_bcd(45, 59).unwrap(), (5, 4));
        assert_eq!(DS3231DateTime::make_bcd(59, 59).unwrap(), (9, 5));
    }

    #[test]
    fn test_make_bcd_invalid() {
        // Test values exceeding max_value
        assert!(matches!(
            DS3231DateTime::make_bcd(60, 59),
            Err(DS3231DateTimeError::InvalidDateTime)
        ));
        assert!(matches!(
            DS3231DateTime::make_bcd(99, 59),
            Err(DS3231DateTimeError::InvalidDateTime)
        ));
        assert!(matches!(
            DS3231DateTime::make_bcd(32, 31),
            Err(DS3231DateTimeError::InvalidDateTime)
        ));
        assert!(matches!(
            DS3231DateTime::make_bcd(13, 12),
            Err(DS3231DateTimeError::InvalidDateTime)
        ));
    }

    #[test]
    fn test_from_datetime_and_into_datetime_roundtrip() {
        let dt = NaiveDate::from_ymd_opt(2024, 3, 14)
            .unwrap()
            .and_hms_opt(15, 30, 0)
            .unwrap();
        let raw = DS3231DateTime::from_datetime(&dt, TimeRepresentation::TwentyFourHour).unwrap();
        let dt2 = raw.into_datetime().unwrap();
        assert_eq!(dt, dt2);
    }

    #[test]
    fn test_from_datetime_century_flag() {
        let dt = NaiveDate::from_ymd_opt(2099, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let raw = DS3231DateTime::from_datetime(&dt, TimeRepresentation::TwentyFourHour).unwrap();
        // The month register should have the century bit set for years >= 2100
        assert_eq!(raw.month.century(), false);
        let dt2 = NaiveDate::from_ymd_opt(2100, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let raw2 = DS3231DateTime::from_datetime(&dt2, TimeRepresentation::TwentyFourHour).unwrap();
        assert_eq!(raw2.month.century(), true);
    }

    #[test]
    fn test_from_datetime_year_too_early() {
        let dt = NaiveDate::from_ymd_opt(1999, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let err =
            DS3231DateTime::from_datetime(&dt, TimeRepresentation::TwentyFourHour).unwrap_err();
        assert!(matches!(err, DS3231DateTimeError::YearNotAfter1999));
    }

    #[test]
    fn test_from_datetime_year_too_late() {
        let dt = NaiveDate::from_ymd_opt(2200, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let err =
            DS3231DateTime::from_datetime(&dt, TimeRepresentation::TwentyFourHour).unwrap_err();
        assert!(matches!(err, DS3231DateTimeError::YearNotBefore2200));
    }

    #[test]
    fn test_from_and_into_bcd_array() {
        let dt = NaiveDate::from_ymd_opt(2024, 3, 14)
            .unwrap()
            .and_hms_opt(15, 30, 0)
            .unwrap();
        let raw = DS3231DateTime::from_datetime(&dt, TimeRepresentation::TwentyFourHour).unwrap();
        let arr: [u8; 7] = (&raw).into();
        let raw2 = DS3231DateTime::from(arr);
        let dt2 = raw2.into_datetime().unwrap();
        assert_eq!(dt, dt2);
    }

    #[test]
    fn test_invalid_bcd_to_datetime() {
        // Invalid BCD values for month (0x13 = 19 in decimal)
        let arr = [0x00, 0x00, 0x00, 0x01, 0x01, 0x13, 0x24];
        let raw = DS3231DateTime::from(arr);
        let result = raw.into_datetime();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DS3231DateTimeError::InvalidDateTime
        ));
    }

    #[test]
    fn test_valid_edge_cases() {
        // Test maximum valid values
        let dt = NaiveDate::from_ymd_opt(2099, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let result = DS3231DateTime::from_datetime(&dt, TimeRepresentation::TwentyFourHour);
        assert!(result.is_ok());

        // Test minimum valid values
        let dt = NaiveDate::from_ymd_opt(2000, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let result = DS3231DateTime::from_datetime(&dt, TimeRepresentation::TwentyFourHour);
        assert!(result.is_ok());
    }

    #[test]
    fn test_twelve_hour_mode() {
        // Test PM time (1 PM = 13:00)
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(13, 0, 0)
            .unwrap();
        let raw = DS3231DateTime::from_datetime(&dt, TimeRepresentation::TwelveHour).unwrap();
        assert_eq!(
            raw.hours.time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(
            raw.hours.pm_or_twenty_hours(),
            1,
            "PM flag should be set for afternoon time"
        );
        assert_eq!(
            raw.hours.hours(),
            1,
            "Hour should be 1 for 13:00 in 12-hour mode"
        );

        // Test AM time (11 AM = 11:00)
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(11, 0, 0)
            .unwrap();
        let raw = DS3231DateTime::from_datetime(&dt, TimeRepresentation::TwelveHour).unwrap();
        assert_eq!(
            raw.hours.time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(
            raw.hours.pm_or_twenty_hours(),
            0,
            "PM flag should not be set for morning time"
        );
        assert_eq!(
            raw.hours.hours(),
            1,
            "Hour should be 1 for 11:00 in 12-hour mode"
        );
        assert_eq!(raw.hours.ten_hours(), 1, "Ten hours should be 1 for 11:00");
    }
}
