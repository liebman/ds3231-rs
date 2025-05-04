//! DateTime conversion and register utilities for the DS3231 RTC.
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
    pub(crate) fn from_datetime(
        datetime: &NaiveDateTime,
        time_representation: TimeRepresentation,
    ) -> Result<Self, DS3231DateTimeError> {
        let seconds = {
            let ones = (datetime.second() % 10) as u8;
            let tens = (datetime.second() / 10) as u8;
            let mut value = Seconds::default();
            value.set_seconds(ones);
            value.set_ten_seconds(tens);
            value
        };
        let minutes = {
            let ones = (datetime.minute() % 10) as u8;
            let tens = (datetime.minute() / 10) as u8;
            let mut value = Minutes::default();
            value.set_minutes(ones);
            value.set_ten_minutes(tens);
            value
        };
        let hours = {
            let ones = (datetime.hour() % 10) as u8;
            let ten = (datetime.hour() / 10) as u8 & 0x01;
            let twenty = ((datetime.hour() / 10) as u8 >> 1) & 0x01;
            let mut value = Hours::default();
            value.set_time_representation(time_representation);
            value.set_hours(ones);
            value.set_ten_hours(ten);
            value.set_pm_or_twenty_hours(twenty);
            value
        };
        let day = {
            let mut value = Day::default();
            value.set_day(datetime.weekday().num_days_from_sunday() as u8);
            value
        };
        let date = {
            let ones = (datetime.day() % 10) as u8;
            let tens = (datetime.day() / 10) as u8;
            let mut value = Date::default();
            value.set_date(ones);
            value.set_ten_date(tens);
            value
        };
        let mut month = {
            let ones = (datetime.month() % 10) as u8;
            let tens = (datetime.month() / 10) as u8;
            let mut value = Month::default();
            value.set_month(ones);
            value.set_ten_month(tens);
            value
        };
        let year = {
            let year: i32 = datetime.year() - 2000;
            if year > 199 {
                #[cfg(any(feature = "log", feature = "defmt"))]
                error!("Year {} is too late! must be before 2200", datetime.year());
                return Err(DS3231DateTimeError::YearNotBefore2200);
            }
            if year < 0 {
                #[cfg(any(feature = "log", feature = "defmt"))]
                error!(
                    "Year {} is too early! must be greater than 1999",
                    datetime.year()
                );
                return Err(DS3231DateTimeError::YearNotAfter1999);
            }
            let mut year = year.unsigned_abs() as u8;
            #[cfg(any(feature = "log", feature = "defmt"))]
            debug!("unsigned raw year={}", year);
            if year > 99 {
                year -= 100;
                month.set_century(true);
            }
            #[cfg(any(feature = "log", feature = "defmt"))]
            debug!("year={} month={:?}", year, month);
            let ones = year % 10;
            let tens = year / 10;
            #[cfg(any(feature = "log", feature = "defmt"))]
            debug!("ones={} tens={}", ones, tens);
            let mut value = Year::default();
            value.set_year(ones);
            value.set_ten_year(tens);
            value
        };
        #[cfg(any(feature = "log", feature = "defmt"))]
        debug!("year={:?}", year);
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

        let year =
            2000 + (10 * u32::from(self.year.ten_year()) + u32::from(self.year.year())) as i32;
        let month = 10 * u32::from(self.month.ten_month()) + u32::from(self.month.month());
        let date = 10 * u32::from(self.date.ten_date()) + u32::from(self.date.date());

        // Validate the date components before creating NaiveDateTime
        NaiveDate::from_ymd_opt(year, month, date)
            .and_then(|d| d.and_hms_opt(hours, minutes, seconds))
            .ok_or(DS3231DateTimeError::InvalidDateTime)
    }
}

impl TryInto<NaiveDateTime> for DS3231DateTime {
    type Error = DS3231DateTimeError;

    fn try_into(self) -> Result<NaiveDateTime, Self::Error> {
        self.into_datetime()
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
}
