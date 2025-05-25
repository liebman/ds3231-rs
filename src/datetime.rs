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
    pub(crate) fn make_bcd(value: u32, max_value: u32) -> Result<(u8, u8), DS3231DateTimeError> {
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

    pub(crate) fn convert_hours(
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
                let ten_hours = u8::from((10..20).contains(&hour));
                let twenty_hours = u8::from(hour >= 20);
                value.set_hours(ones);
                value.set_ten_hours(ten_hours);
                value.set_pm_or_twenty_hours(twenty_hours);
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
            error!("Year {} is too late! must be before 2200", year);
            return Err(DS3231DateTimeError::YearNotBefore2200);
        }
        if year < 2000 {
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
                let is_pm = self.hours.pm_or_twenty_hours() != 0;
                match (hours, is_pm) {
                    (12, false) => 0,    // 12 AM = 0:xx
                    (12, true) => 12,    // 12 PM = 12:xx
                    (h, false) => h,     // 1-11 AM = 1-11:xx
                    (h, true) => h + 12, // 1-11 PM = 13-23:xx
                }
            }
        };
        debug!(
            "raw_hour={:?} h={} m={} s={}",
            self.hours, hours, minutes, seconds
        );

        let year_offset = 10 * u32::from(self.year.ten_year()) + u32::from(self.year.year());
        let century_offset = if self.month.century() { 100 } else { 0 };
        let year = 2000_i32
            + i32::try_from(year_offset + century_offset)
                .map_err(|_| DS3231DateTimeError::InvalidDateTime)?;
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
        core::assert_eq!(dt, dt2);
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
        core::assert_eq!(dt, dt2);
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

    #[test]
    fn test_convert_functions_coverage() {
        // Test convert_seconds edge cases
        assert!(DS3231DateTime::convert_seconds(60).is_err());
        assert!(DS3231DateTime::convert_seconds(0).is_ok());
        assert!(DS3231DateTime::convert_seconds(59).is_ok());

        // Test convert_minutes edge cases
        assert!(DS3231DateTime::convert_minutes(60).is_err());
        assert!(DS3231DateTime::convert_minutes(0).is_ok());
        assert!(DS3231DateTime::convert_minutes(59).is_ok());

        // Test convert_day edge cases
        assert!(DS3231DateTime::convert_day(7).is_err());
        assert!(DS3231DateTime::convert_day(0).is_ok());
        assert!(DS3231DateTime::convert_day(6).is_ok());

        // Test convert_date edge cases
        assert!(DS3231DateTime::convert_date(32).is_err());
        assert!(DS3231DateTime::convert_date(1).is_ok());
        assert!(DS3231DateTime::convert_date(31).is_ok());

        // Test convert_month edge cases
        assert!(DS3231DateTime::convert_month(13).is_err());
        assert!(DS3231DateTime::convert_month(1).is_ok());
        assert!(DS3231DateTime::convert_month(12).is_ok());
    }

    #[test]
    fn test_convert_hours_comprehensive() {
        // Test 24-hour mode edge cases
        let hours_0 = DS3231DateTime::convert_hours(0, TimeRepresentation::TwentyFourHour).unwrap();
        assert_eq!(
            hours_0.time_representation(),
            TimeRepresentation::TwentyFourHour
        );
        assert_eq!(hours_0.hours(), 0);
        assert_eq!(hours_0.ten_hours(), 0);
        assert_eq!(hours_0.pm_or_twenty_hours(), 0);

        let hours_23 =
            DS3231DateTime::convert_hours(23, TimeRepresentation::TwentyFourHour).unwrap();
        assert_eq!(
            hours_23.time_representation(),
            TimeRepresentation::TwentyFourHour
        );
        assert_eq!(hours_23.hours(), 3);
        assert_eq!(hours_23.ten_hours(), 0); // For 23: ten_hours bit is 0 (only set for 10-19)
        assert_eq!(hours_23.pm_or_twenty_hours(), 1); // twenty = (23/10) >> 1 & 0x01 = 1

        // Test 12-hour mode edge cases
        let hours_12am = DS3231DateTime::convert_hours(0, TimeRepresentation::TwelveHour).unwrap();
        assert_eq!(
            hours_12am.time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(hours_12am.hours(), 2);
        assert_eq!(hours_12am.ten_hours(), 1); // 12 AM: tens=1, ones=2
        assert_eq!(hours_12am.pm_or_twenty_hours(), 0); // AM

        let hours_12pm = DS3231DateTime::convert_hours(12, TimeRepresentation::TwelveHour).unwrap();
        assert_eq!(
            hours_12pm.time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(hours_12pm.hours(), 2);
        assert_eq!(hours_12pm.ten_hours(), 1); // 12 PM: tens=1, ones=2
        assert_eq!(hours_12pm.pm_or_twenty_hours(), 1); // PM

        let hours_1pm = DS3231DateTime::convert_hours(13, TimeRepresentation::TwelveHour).unwrap();
        assert_eq!(
            hours_1pm.time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(hours_1pm.hours(), 1);
        assert_eq!(hours_1pm.ten_hours(), 0);
        assert_eq!(hours_1pm.pm_or_twenty_hours(), 1); // PM

        // Test invalid hours
        assert!(DS3231DateTime::convert_hours(24, TimeRepresentation::TwentyFourHour).is_err());
        assert!(DS3231DateTime::convert_hours(24, TimeRepresentation::TwelveHour).is_err());
    }

    #[test]
    fn test_convert_year_comprehensive() {
        // Test year 2000
        let (year_2000, century_2000) = DS3231DateTime::convert_year(2000).unwrap();
        assert_eq!(year_2000.year(), 0);
        assert_eq!(year_2000.ten_year(), 0);
        assert!(!century_2000);

        // Test year 2099
        let (year_2099, century_2099) = DS3231DateTime::convert_year(2099).unwrap();
        assert_eq!(year_2099.year(), 9);
        assert_eq!(year_2099.ten_year(), 9);
        assert!(!century_2099);

        // Test year 2100
        let (year_2100, century_2100) = DS3231DateTime::convert_year(2100).unwrap();
        assert_eq!(year_2100.year(), 0);
        assert_eq!(year_2100.ten_year(), 0);
        assert!(century_2100);

        // Test year 2199
        let (year_2199, century_2199) = DS3231DateTime::convert_year(2199).unwrap();
        assert_eq!(year_2199.year(), 9);
        assert_eq!(year_2199.ten_year(), 9);
        assert!(century_2199);

        // Test invalid years
        assert!(matches!(
            DS3231DateTime::convert_year(1999),
            Err(DS3231DateTimeError::YearNotAfter1999)
        ));
        assert!(matches!(
            DS3231DateTime::convert_year(2200),
            Err(DS3231DateTimeError::YearNotBefore2200)
        ));
    }

    #[test]
    fn test_into_datetime_twelve_hour_mode() {
        // Test 12-hour mode conversion for 2 PM
        let mut raw = DS3231DateTime {
            seconds: Seconds(0x30), // 30 seconds
            minutes: Minutes(0x45), // 45 minutes
            hours: Hours(0x00),     // Will be set properly below
            day: Day(0x04),         // Thursday
            date: Date(0x14),       // 14th
            month: Month(0x03),     // March
            year: Year(0x24),       // 2024
        };
        raw.hours
            .set_time_representation(TimeRepresentation::TwelveHour);
        raw.hours.set_pm_or_twenty_hours(1); // PM
        raw.hours.set_ten_hours(0); // For hour 2, tens digit is 0
        raw.hours.set_hours(2); // Hour 2

        let dt = raw.into_datetime().unwrap();
        assert_eq!(dt.hour(), 14); // 2 PM = 14:00 in 24-hour
        assert_eq!(dt.minute(), 45);
        assert_eq!(dt.second(), 30);
    }

    #[test]
    fn test_invalid_bcd_values() {
        // Test invalid seconds BCD
        let invalid_seconds = DS3231DateTime {
            seconds: Seconds(0x6A), // Invalid BCD (6A = 106 decimal, but should be max 59)
            minutes: Minutes(0x00),
            hours: Hours(0x00),
            day: Day(0x01),
            date: Date(0x01),
            month: Month(0x01),
            year: Year(0x00),
        };
        assert!(invalid_seconds.into_datetime().is_err());

        // Test invalid minutes BCD
        let invalid_minutes = DS3231DateTime {
            seconds: Seconds(0x00),
            minutes: Minutes(0x6A), // Invalid BCD
            hours: Hours(0x00),
            day: Day(0x01),
            date: Date(0x01),
            month: Month(0x01),
            year: Year(0x00),
        };
        assert!(invalid_minutes.into_datetime().is_err());

        // Test invalid date
        let invalid_date = DS3231DateTime {
            seconds: Seconds(0x00),
            minutes: Minutes(0x00),
            hours: Hours(0x00),
            day: Day(0x01),
            date: Date(0x32), // 32nd day doesn't exist
            month: Month(0x01),
            year: Year(0x00),
        };
        assert!(invalid_date.into_datetime().is_err());
    }

    #[test]
    fn test_array_conversions() {
        let dt = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(10, 25, 45)
            .unwrap();
        let raw = DS3231DateTime::from_datetime(&dt, TimeRepresentation::TwentyFourHour).unwrap();

        // Test conversion to array
        let arr: [u8; 7] = (&raw).into();

        // Test conversion back from array
        let raw2 = DS3231DateTime::from(arr);

        // Should be identical
        assert_eq!(raw, raw2);

        // Should convert back to same datetime
        let dt2 = raw2.into_datetime().unwrap();
        assert_eq!(dt, dt2);
    }

    #[test]
    fn test_error_debug_formatting() {
        extern crate alloc;

        // Test Debug formatting for error types
        let invalid_error = DS3231DateTimeError::InvalidDateTime;
        let debug_str = alloc::format!("{:?}", invalid_error);
        assert!(debug_str.contains("InvalidDateTime"));

        let year_early_error = DS3231DateTimeError::YearNotAfter1999;
        let debug_str = alloc::format!("{:?}", year_early_error);
        assert!(debug_str.contains("YearNotAfter1999"));

        let year_late_error = DS3231DateTimeError::YearNotBefore2200;
        let debug_str = alloc::format!("{:?}", year_late_error);
        assert!(debug_str.contains("YearNotBefore2200"));
    }

    #[test]
    fn test_leap_year_handling() {
        // Test leap year (2024)
        let leap_year_dt = NaiveDate::from_ymd_opt(2024, 2, 29)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let raw = DS3231DateTime::from_datetime(&leap_year_dt, TimeRepresentation::TwentyFourHour)
            .unwrap();
        let converted_back = raw.into_datetime().unwrap();
        assert_eq!(leap_year_dt, converted_back);

        // Test non-leap year boundary
        let non_leap_year_dt = NaiveDate::from_ymd_opt(2023, 2, 28)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let raw =
            DS3231DateTime::from_datetime(&non_leap_year_dt, TimeRepresentation::TwentyFourHour)
                .unwrap();
        let converted_back = raw.into_datetime().unwrap();
        assert_eq!(non_leap_year_dt, converted_back);
    }

    #[test]
    fn test_weekday_conversion() {
        // Test all weekdays
        let sunday = NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(); // Sunday
        let raw = DS3231DateTime::from_datetime(
            &sunday.and_hms_opt(0, 0, 0).unwrap(),
            TimeRepresentation::TwentyFourHour,
        )
        .unwrap();
        assert_eq!(raw.day.day(), 0); // Sunday = 0 in DS3231

        let monday = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap(); // Monday
        let raw = DS3231DateTime::from_datetime(
            &monday.and_hms_opt(0, 0, 0).unwrap(),
            TimeRepresentation::TwentyFourHour,
        )
        .unwrap();
        assert_eq!(raw.day.day(), 1); // Monday = 1 in DS3231

        let saturday = NaiveDate::from_ymd_opt(2024, 3, 16).unwrap(); // Saturday
        let raw = DS3231DateTime::from_datetime(
            &saturday.and_hms_opt(0, 0, 0).unwrap(),
            TimeRepresentation::TwentyFourHour,
        )
        .unwrap();
        assert_eq!(raw.day.day(), 6); // Saturday = 6 in DS3231
    }

    #[test]
    fn test_century_boundary_years() {
        // Test year 2099 -> 2100 transition
        let year_2099 = NaiveDate::from_ymd_opt(2099, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap();
        let raw_2099 =
            DS3231DateTime::from_datetime(&year_2099, TimeRepresentation::TwentyFourHour).unwrap();
        assert!(!raw_2099.month.century());

        let year_2100 = NaiveDate::from_ymd_opt(2100, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let raw_2100 =
            DS3231DateTime::from_datetime(&year_2100, TimeRepresentation::TwentyFourHour).unwrap();
        assert!(raw_2100.month.century());

        // Test roundtrip conversion
        let converted_2099 = raw_2099.into_datetime().unwrap();
        assert_eq!(year_2099, converted_2099);

        let converted_2100 = raw_2100.into_datetime().unwrap();
        assert_eq!(year_2100, converted_2100);
    }
}
