use chrono::DateTime;
use chrono::Datelike;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Timelike;
use chrono::Utc;
use log::debug;
use log::error;

use crate::Date;
use crate::Day;
use crate::Hours;
use crate::Minutes;
use crate::Month;
use crate::Seconds;
use crate::TimeRepresentation;
use crate::Year;

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
        datetime: &DateTime<Utc>,
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
                error!("Year {} is too late! must be before 2200", datetime.year());
                return Err(DS3231DateTimeError::YearNotBefore2200);
            }
            if year < 0 {
                error!(
                    "Year {} is too early! must be greater than 1999",
                    datetime.year()
                );
                return Err(DS3231DateTimeError::YearNotAfter1999);
            }
            let mut year = year.unsigned_abs() as u8;
            debug!("unsigned raw year={}", year);
            if year > 99 {
                year -= 100;
                month.set_century(true);
            }
            debug!("year={} month={:?}", year, month);
            let ones = year % 10;
            let tens = year / 10;
            debug!("ones={} tens={}", ones, tens);
            let mut value = Year::default();
            value.set_year(ones);
            value.set_ten_year(tens);
            value
        };
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
        debug!("raw={:?}", raw);
        Ok(raw)
    }

    pub(crate) fn into_datetime(self) -> Result<DateTime<Utc>, DS3231DateTimeError> {
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
        debug!(
            "raw_hour={:?} h={} m={} s={}",
            self.hours, hours, minutes, seconds
        );
        let ndt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(
                2000 + (10 * u32::from(self.year.ten_year()) + u32::from(self.year.year())) as i32,
                10 * u32::from(self.month.ten_month()) + u32::from(self.month.month()),
                10 * u32::from(self.date.ten_date()) + u32::from(self.date.date()),
            )
            .expect("Invalid date"),
            NaiveTime::from_hms_opt(hours, minutes, seconds).expect("Invalid time"),
        );
        let ts = ndt.and_utc().timestamp();
        match DateTime::from_timestamp(ts, 0) {
            Some(dt) => Ok(dt),
            _ => Err(DS3231DateTimeError::InvalidDateTime),
        }
    }
}

impl TryInto<DateTime<Utc>> for DS3231DateTime {
    type Error = DS3231DateTimeError;

    fn try_into(self) -> Result<DateTime<Utc>, Self::Error> {
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
pub enum DS3231DateTimeError {
    InvalidDateTime,
    YearNotBefore2200,
    YearNotAfter1999,
}
