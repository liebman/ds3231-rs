//! Alarm configuration utilities for the DS3231 RTC.
//!
//! This module provides type-safe alarm configuration for the DS3231's alarm registers.
//! It uses enum-based configurations that clearly express the different alarm modes
//! without the confusion of mixing datetime objects with alarm semantics.
//!
//! # Features
//!
//! - Type-safe configuration of Alarm 1 (with seconds precision)
//! - Type-safe configuration of Alarm 2 (minute precision, triggers at 00 seconds)
//! - Clear separation between time specification and recurrence patterns
//! - Support for both 12-hour and 24-hour time formats
//! - Day-of-week and date-of-month matching
//!
//! # Alarm Types
//!
//! ## Alarm 1 Configurations
//! - `EverySecond` - Triggers every second
//! - `AtSeconds` - Triggers when seconds match
//! - `AtMinutesSeconds` - Triggers when minutes:seconds match
//! - `AtTime` - Triggers when hours:minutes:seconds match (daily)
//! - `AtTimeOnDate` - Triggers at specific time on specific date of month
//! - `AtTimeOnDay` - Triggers at specific time on specific day of week
//!
//! ## Alarm 2 Configurations
//! - `EveryMinute` - Triggers every minute (at 00 seconds)
//! - `AtMinutes` - Triggers when minutes match at 00 seconds
//! - `AtTime` - Triggers when hours:minutes match (at 00 seconds, daily)
//! - `AtTimeOnDate` - Triggers at specific time on specific date of month (at 00 seconds)
//! - `AtTimeOnDay` - Triggers at specific time on specific day of week (at 00 seconds)

use crate::{
    datetime::{DS3231DateTime, DS3231DateTimeError},
    AlarmDayDate, AlarmHours, AlarmMinutes, AlarmSeconds, DayDateSelect, TimeRepresentation,
};

/// Error type for alarm configuration operations.
#[derive(Debug)]
pub enum AlarmError {
    /// Invalid time component value
    InvalidTime(&'static str),
    /// Invalid day of week (must be 1-7)
    InvalidDayOfWeek,
    /// Invalid date of month (must be 1-31)
    InvalidDateOfMonth,
    /// `DateTime` conversion error
    DateTime(DS3231DateTimeError),
}

impl From<DS3231DateTimeError> for AlarmError {
    fn from(e: DS3231DateTimeError) -> Self {
        AlarmError::DateTime(e)
    }
}

/// Alarm 1 specific configurations.
///
/// Alarm 1 supports seconds-level precision and can match against various time components.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Alarm1Config {
    /// Trigger every second (all mask bits set)
    EverySecond,

    /// Trigger when seconds match (A1M1=0, others=1)
    AtSeconds {
        /// Seconds value (0-59)
        seconds: u8,
    },

    /// Trigger when minutes and seconds match (A1M1=0, A1M2=0, others=1)
    AtMinutesSeconds {
        /// Minutes value (0-59)
        minutes: u8,
        /// Seconds value (0-59)
        seconds: u8,
    },

    /// Trigger when hours, minutes, and seconds match (A1M1=0, A1M2=0, A1M3=0, A1M4=1)
    /// This creates a daily alarm at the specified time.
    AtTime {
        /// Hours value (0-23 for 24-hour, 1-12 for 12-hour)
        hours: u8,
        /// Minutes value (0-59)
        minutes: u8,
        /// Seconds value (0-59)
        seconds: u8,
        /// PM flag for 12-hour mode (None for 24-hour, Some(true/false) for 12-hour)
        is_pm: Option<bool>,
    },

    /// Trigger at specific time on specific date of month (all mask bits=0, DY/DT=0)
    AtTimeOnDate {
        /// Hours value (0-23 for 24-hour, 1-12 for 12-hour)
        hours: u8,
        /// Minutes value (0-59)
        minutes: u8,
        /// Seconds value (0-59)
        seconds: u8,
        /// Date of month (1-31)
        date: u8,
        /// PM flag for 12-hour mode (None for 24-hour, Some(true/false) for 12-hour)
        is_pm: Option<bool>,
    },

    /// Trigger at specific time on specific day of week (all mask bits=0, DY/DT=1)
    AtTimeOnDay {
        /// Hours value (0-23 for 24-hour, 1-12 for 12-hour)
        hours: u8,
        /// Minutes value (0-59)
        minutes: u8,
        /// Seconds value (0-59)
        seconds: u8,
        /// Day of week (1-7, where 1=Sunday)
        day: u8,
        /// PM flag for 12-hour mode (None for 24-hour, Some(true/false) for 12-hour)
        is_pm: Option<bool>,
    },
}

/// Alarm 2 specific configurations.
///
/// Alarm 2 has no seconds register and always triggers at 00 seconds of the matching minute.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Alarm2Config {
    /// Trigger every minute at 00 seconds (all mask bits set)
    EveryMinute,

    /// Trigger when minutes match at 00 seconds (A2M2=0, others=1)
    AtMinutes {
        /// Minutes value (0-59)
        minutes: u8,
    },

    /// Trigger when hours and minutes match at 00 seconds (A2M2=0, A2M3=0, A2M4=1)
    /// This creates a daily alarm at the specified time.
    AtTime {
        /// Hours value (0-23 for 24-hour, 1-12 for 12-hour)
        hours: u8,
        /// Minutes value (0-59)
        minutes: u8,
        /// PM flag for 12-hour mode (None for 24-hour, Some(true/false) for 12-hour)
        is_pm: Option<bool>,
    },

    /// Trigger at specific time on specific date of month at 00 seconds (all mask bits=0, DY/DT=0)
    AtTimeOnDate {
        /// Hours value (0-23 for 24-hour, 1-12 for 12-hour)
        hours: u8,
        /// Minutes value (0-59)
        minutes: u8,
        /// Date of month (1-31)
        date: u8,
        /// PM flag for 12-hour mode (None for 24-hour, Some(true/false) for 12-hour)
        is_pm: Option<bool>,
    },

    /// Trigger at specific time on specific day of week at 00 seconds (all mask bits=0, DY/DT=1)
    AtTimeOnDay {
        /// Hours value (0-23 for 24-hour, 1-12 for 12-hour)
        hours: u8,
        /// Minutes value (0-59)
        minutes: u8,
        /// Day of week (1-7, where 1=Sunday)
        day: u8,
        /// PM flag for 12-hour mode (None for 24-hour, Some(true/false) for 12-hour)
        is_pm: Option<bool>,
    },
}

impl Alarm1Config {
    /// Validates the alarm configuration and returns any errors.
    ///
    /// # Errors
    ///
    /// Returns an error if any time component is out of valid range.
    pub fn validate(&self) -> Result<(), AlarmError> {
        match self {
            Alarm1Config::EverySecond => Ok(()),

            Alarm1Config::AtSeconds { seconds } => {
                if *seconds > 59 {
                    Err(AlarmError::InvalidTime("seconds must be 0-59"))
                } else {
                    Ok(())
                }
            }

            Alarm1Config::AtMinutesSeconds { minutes, seconds } => {
                if *minutes > 59 {
                    Err(AlarmError::InvalidTime("minutes must be 0-59"))
                } else if *seconds > 59 {
                    Err(AlarmError::InvalidTime("seconds must be 0-59"))
                } else {
                    Ok(())
                }
            }

            Alarm1Config::AtTime {
                hours,
                minutes,
                seconds,
                is_pm,
            } => Self::validate_time(*hours, *minutes, *seconds, *is_pm),

            Alarm1Config::AtTimeOnDate {
                hours,
                minutes,
                seconds,
                date,
                is_pm,
            } => {
                Self::validate_time(*hours, *minutes, *seconds, *is_pm)?;
                if *date == 0 || *date > 31 {
                    Err(AlarmError::InvalidDateOfMonth)
                } else {
                    Ok(())
                }
            }

            Alarm1Config::AtTimeOnDay {
                hours,
                minutes,
                seconds,
                day,
                is_pm,
            } => {
                Self::validate_time(*hours, *minutes, *seconds, *is_pm)?;
                if *day == 0 || *day > 7 {
                    Err(AlarmError::InvalidDayOfWeek)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn validate_time(
        hours: u8,
        minutes: u8,
        seconds: u8,
        is_pm: Option<bool>,
    ) -> Result<(), AlarmError> {
        if minutes > 59 {
            return Err(AlarmError::InvalidTime("minutes must be 0-59"));
        }
        if seconds > 59 {
            return Err(AlarmError::InvalidTime("seconds must be 0-59"));
        }

        match is_pm {
            None => {
                // 24-hour mode
                if hours > 23 {
                    Err(AlarmError::InvalidTime(
                        "hours must be 0-23 in 24-hour mode",
                    ))
                } else {
                    Ok(())
                }
            }
            Some(_) => {
                // 12-hour mode
                if hours == 0 || hours > 12 {
                    Err(AlarmError::InvalidTime(
                        "hours must be 1-12 in 12-hour mode",
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl Alarm2Config {
    /// Validates the alarm configuration and returns any errors.
    ///
    /// # Errors
    ///
    /// Returns an error if any time component is out of valid range.
    pub fn validate(&self) -> Result<(), AlarmError> {
        match self {
            Alarm2Config::EveryMinute => Ok(()),

            Alarm2Config::AtMinutes { minutes } => {
                if *minutes > 59 {
                    Err(AlarmError::InvalidTime("minutes must be 0-59"))
                } else {
                    Ok(())
                }
            }

            Alarm2Config::AtTime {
                hours,
                minutes,
                is_pm,
            } => Self::validate_time(*hours, *minutes, *is_pm),

            Alarm2Config::AtTimeOnDate {
                hours,
                minutes,
                date,
                is_pm,
            } => {
                Self::validate_time(*hours, *minutes, *is_pm)?;
                if *date == 0 || *date > 31 {
                    Err(AlarmError::InvalidDateOfMonth)
                } else {
                    Ok(())
                }
            }

            Alarm2Config::AtTimeOnDay {
                hours,
                minutes,
                day,
                is_pm,
            } => {
                Self::validate_time(*hours, *minutes, *is_pm)?;
                if *day == 0 || *day > 7 {
                    Err(AlarmError::InvalidDayOfWeek)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn validate_time(hours: u8, minutes: u8, is_pm: Option<bool>) -> Result<(), AlarmError> {
        if minutes > 59 {
            return Err(AlarmError::InvalidTime("minutes must be 0-59"));
        }

        match is_pm {
            None => {
                // 24-hour mode
                if hours > 23 {
                    Err(AlarmError::InvalidTime(
                        "hours must be 0-23 in 24-hour mode",
                    ))
                } else {
                    Ok(())
                }
            }
            Some(_) => {
                // 12-hour mode
                if hours == 0 || hours > 12 {
                    Err(AlarmError::InvalidTime(
                        "hours must be 1-12 in 12-hour mode",
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Internal representation of DS3231 Alarm 1 registers.
///
/// This struct models the 4 alarm 1 registers of the DS3231, using strongly-typed bitfield wrappers for each field.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DS3231Alarm1 {
    seconds: AlarmSeconds,
    minutes: AlarmMinutes,
    hours: AlarmHours,
    day_date: AlarmDayDate,
}

/// Creates configured time components (minutes and hours) for both alarm types
fn create_alarm_time_components(
    hour: u8,
    minute: u8,
    is_pm: Option<bool>,
) -> Result<(AlarmMinutes, AlarmHours), AlarmError> {
    // Create minutes
    let (min_ones, min_tens) = DS3231DateTime::make_bcd(u32::from(minute), 59)?;
    let mut minutes = AlarmMinutes::default();
    minutes.set_minutes(min_ones);
    minutes.set_ten_minutes(min_tens);

    // Create hours based on format
    let mut hours = AlarmHours::default();
    match is_pm {
        None => {
            // 24-hour mode
            hours.set_time_representation(TimeRepresentation::TwentyFourHour);
            let hour_reg =
                DS3231DateTime::convert_hours(u32::from(hour), TimeRepresentation::TwentyFourHour)?;
            hours.set_hours(hour_reg.hours());
            hours.set_ten_hours(hour_reg.ten_hours());
            hours.set_pm_or_twenty_hours(hour_reg.pm_or_twenty_hours());
        }
        Some(pm) => {
            // 12-hour mode
            hours.set_time_representation(TimeRepresentation::TwelveHour);
            let hour_reg =
                DS3231DateTime::convert_hours(u32::from(hour), TimeRepresentation::TwelveHour)?;
            hours.set_hours(hour_reg.hours());
            hours.set_ten_hours(hour_reg.ten_hours());
            hours.set_pm_or_twenty_hours(u8::from(pm));
        }
    }

    Ok((minutes, hours))
}

/// Creates configured day/date component for both alarm types
fn create_alarm_day_date_component(
    day_or_date: u8,
    is_day: bool,
) -> Result<AlarmDayDate, AlarmError> {
    let mut day_date = AlarmDayDate::default();

    if is_day {
        day_date.set_day_date_select(DayDateSelect::Day);
        day_date.set_day_or_date(day_or_date);
    } else {
        day_date.set_day_date_select(DayDateSelect::Date);
        let (date_ones, date_tens) = DS3231DateTime::make_bcd(u32::from(day_or_date), 31)?;
        day_date.set_day_or_date(date_ones);
        day_date.set_ten_date(date_tens);
    }

    Ok(day_date)
}

impl DS3231Alarm1 {
    /// Creates an Alarm 1 register configuration from an `Alarm1Config`.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or contains out-of-range values.
    pub fn from_config(config: &Alarm1Config) -> Result<Self, AlarmError> {
        config.validate()?;

        let mut alarm = Self {
            seconds: AlarmSeconds::default(),
            minutes: AlarmMinutes::default(),
            hours: AlarmHours::default(),
            day_date: AlarmDayDate::default(),
        };

        match config {
            Alarm1Config::EverySecond => {
                Self::configure_every_second(&mut alarm);
            }

            Alarm1Config::AtSeconds { seconds: sec } => {
                Self::configure_at_seconds(&mut alarm, *sec)?;
            }

            Alarm1Config::AtMinutesSeconds {
                minutes: min,
                seconds: sec,
            } => {
                Self::configure_at_minutes_seconds(&mut alarm, *min, *sec)?;
            }

            Alarm1Config::AtTime {
                hours: hr,
                minutes: min,
                seconds: sec,
                is_pm,
            } => {
                Self::configure_at_time(&mut alarm, *hr, *min, *sec, *is_pm)?;
            }

            Alarm1Config::AtTimeOnDate {
                hours: hr,
                minutes: min,
                seconds: sec,
                date,
                is_pm,
            } => {
                Self::configure_at_time_on_date(&mut alarm, *hr, *min, *sec, *date, *is_pm)?;
            }

            Alarm1Config::AtTimeOnDay {
                hours: hr,
                minutes: min,
                seconds: sec,
                day,
                is_pm,
            } => {
                Self::configure_at_time_on_day(&mut alarm, *hr, *min, *sec, *day, *is_pm)?;
            }
        }

        Ok(alarm)
    }

    fn configure_every_second(alarm: &mut Self) {
        alarm.seconds.set_alarm_mask1(true);
        alarm.minutes.set_alarm_mask2(true);
        alarm.hours.set_alarm_mask3(true);
        alarm.day_date.set_alarm_mask4(true);
    }

    fn configure_at_seconds(alarm: &mut Self, sec: u8) -> Result<(), AlarmError> {
        let (sec_ones, sec_tens) = DS3231DateTime::make_bcd(u32::from(sec), 59)?;
        alarm.seconds.set_seconds(sec_ones);
        alarm.seconds.set_ten_seconds(sec_tens);
        alarm.seconds.set_alarm_mask1(false);
        alarm.minutes.set_alarm_mask2(true);
        alarm.hours.set_alarm_mask3(true);
        alarm.day_date.set_alarm_mask4(true);
        Ok(())
    }

    fn configure_at_minutes_seconds(alarm: &mut Self, min: u8, sec: u8) -> Result<(), AlarmError> {
        let (sec_ones, sec_tens) = DS3231DateTime::make_bcd(u32::from(sec), 59)?;
        alarm.seconds.set_seconds(sec_ones);
        alarm.seconds.set_ten_seconds(sec_tens);
        alarm.seconds.set_alarm_mask1(false);

        let (min_ones, min_tens) = DS3231DateTime::make_bcd(u32::from(min), 59)?;
        alarm.minutes.set_minutes(min_ones);
        alarm.minutes.set_ten_minutes(min_tens);
        alarm.minutes.set_alarm_mask2(false);

        alarm.hours.set_alarm_mask3(true);
        alarm.day_date.set_alarm_mask4(true);
        Ok(())
    }

    fn configure_at_time(
        alarm: &mut Self,
        hr: u8,
        min: u8,
        sec: u8,
        is_pm: Option<bool>,
    ) -> Result<(), AlarmError> {
        Self::set_time_components(
            &mut alarm.seconds,
            &mut alarm.minutes,
            &mut alarm.hours,
            hr,
            min,
            sec,
            is_pm,
        )?;
        alarm.seconds.set_alarm_mask1(false);
        alarm.minutes.set_alarm_mask2(false);
        alarm.hours.set_alarm_mask3(false);
        alarm.day_date.set_alarm_mask4(true);
        Ok(())
    }

    fn configure_at_time_on_date(
        alarm: &mut Self,
        hr: u8,
        min: u8,
        sec: u8,
        date: u8,
        is_pm: Option<bool>,
    ) -> Result<(), AlarmError> {
        Self::set_time_components(
            &mut alarm.seconds,
            &mut alarm.minutes,
            &mut alarm.hours,
            hr,
            min,
            sec,
            is_pm,
        )?;
        alarm.seconds.set_alarm_mask1(false);
        alarm.minutes.set_alarm_mask2(false);
        alarm.hours.set_alarm_mask3(false);
        alarm.day_date.set_alarm_mask4(false);

        alarm.day_date = create_alarm_day_date_component(date, false)?;
        Ok(())
    }

    fn configure_at_time_on_day(
        alarm: &mut Self,
        hr: u8,
        min: u8,
        sec: u8,
        day: u8,
        is_pm: Option<bool>,
    ) -> Result<(), AlarmError> {
        Self::set_time_components(
            &mut alarm.seconds,
            &mut alarm.minutes,
            &mut alarm.hours,
            hr,
            min,
            sec,
            is_pm,
        )?;
        alarm.seconds.set_alarm_mask1(false);
        alarm.minutes.set_alarm_mask2(false);
        alarm.hours.set_alarm_mask3(false);
        alarm.day_date.set_alarm_mask4(false);

        alarm.day_date = create_alarm_day_date_component(day, true)?;
        Ok(())
    }

    fn set_time_components(
        seconds: &mut AlarmSeconds,
        minutes: &mut AlarmMinutes,
        hours: &mut AlarmHours,
        hour: u8,
        minute: u8,
        second: u8,
        is_pm: Option<bool>,
    ) -> Result<(), AlarmError> {
        // Set seconds
        let (sec_ones, sec_tens) = DS3231DateTime::make_bcd(u32::from(second), 59)?;
        seconds.set_seconds(sec_ones);
        seconds.set_ten_seconds(sec_tens);

        // Use shared helper for minutes and hours
        let (new_minutes, new_hours) = create_alarm_time_components(hour, minute, is_pm)?;
        *minutes = new_minutes;
        *hours = new_hours;
        Ok(())
    }

    /// Gets the alarm seconds register
    #[must_use]
    pub fn seconds(&self) -> AlarmSeconds {
        self.seconds
    }

    /// Gets the alarm minutes register
    #[must_use]
    pub fn minutes(&self) -> AlarmMinutes {
        self.minutes
    }

    /// Gets the alarm hours register
    #[must_use]
    pub fn hours(&self) -> AlarmHours {
        self.hours
    }

    /// Gets the alarm day/date register
    #[must_use]
    pub fn day_date(&self) -> AlarmDayDate {
        self.day_date
    }

    /// Creates an Alarm 1 configuration from existing register values.
    #[must_use]
    pub fn from_registers(
        seconds: AlarmSeconds,
        minutes: AlarmMinutes,
        hours: AlarmHours,
        day_date: AlarmDayDate,
    ) -> Self {
        DS3231Alarm1 {
            seconds,
            minutes,
            hours,
            day_date,
        }
    }
}

/// Internal representation of DS3231 Alarm 2 registers.
///
/// This struct models the 3 alarm 2 registers of the DS3231 (no seconds register).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DS3231Alarm2 {
    minutes: AlarmMinutes,
    hours: AlarmHours,
    day_date: AlarmDayDate,
}

impl DS3231Alarm2 {
    /// Creates an Alarm 2 register configuration from an `Alarm2Config`.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or contains out-of-range values.
    pub fn from_config(config: &Alarm2Config) -> Result<Self, AlarmError> {
        config.validate()?;

        let mut minutes = AlarmMinutes::default();
        let mut hours = AlarmHours::default();
        let mut day_date = AlarmDayDate::default();

        match config {
            Alarm2Config::EveryMinute => {
                // All mask bits set
                minutes.set_alarm_mask2(true);
                hours.set_alarm_mask3(true);
                day_date.set_alarm_mask4(true);
            }

            Alarm2Config::AtMinutes { minutes: min } => {
                let (min_ones, min_tens) = DS3231DateTime::make_bcd(u32::from(*min), 59)?;
                minutes.set_minutes(min_ones);
                minutes.set_ten_minutes(min_tens);
                minutes.set_alarm_mask2(false);
                hours.set_alarm_mask3(true);
                day_date.set_alarm_mask4(true);
            }

            Alarm2Config::AtTime {
                hours: hr,
                minutes: min,
                is_pm,
            } => {
                Self::set_time_components(&mut minutes, &mut hours, *hr, *min, *is_pm)?;
                minutes.set_alarm_mask2(false);
                hours.set_alarm_mask3(false);
                day_date.set_alarm_mask4(true); // Don't match day/date
            }

            Alarm2Config::AtTimeOnDate {
                hours: hr,
                minutes: min,
                date,
                is_pm,
            } => {
                Self::set_time_components(&mut minutes, &mut hours, *hr, *min, *is_pm)?;
                minutes.set_alarm_mask2(false);
                hours.set_alarm_mask3(false);
                day_date.set_alarm_mask4(false);

                day_date = create_alarm_day_date_component(*date, false)?;
            }

            Alarm2Config::AtTimeOnDay {
                hours: hr,
                minutes: min,
                day,
                is_pm,
            } => {
                Self::set_time_components(&mut minutes, &mut hours, *hr, *min, *is_pm)?;
                minutes.set_alarm_mask2(false);
                hours.set_alarm_mask3(false);
                day_date.set_alarm_mask4(false);

                day_date = create_alarm_day_date_component(*day, true)?;
            }
        }

        Ok(DS3231Alarm2 {
            minutes,
            hours,
            day_date,
        })
    }

    fn set_time_components(
        minutes: &mut AlarmMinutes,
        hours: &mut AlarmHours,
        hour: u8,
        minute: u8,
        is_pm: Option<bool>,
    ) -> Result<(), AlarmError> {
        // Use shared helper for minutes and hours
        let (new_minutes, new_hours) = create_alarm_time_components(hour, minute, is_pm)?;
        *minutes = new_minutes;
        *hours = new_hours;
        Ok(())
    }

    /// Gets the alarm minutes register
    #[must_use]
    pub fn minutes(&self) -> AlarmMinutes {
        self.minutes
    }

    /// Gets the alarm hours register
    #[must_use]
    pub fn hours(&self) -> AlarmHours {
        self.hours
    }

    /// Gets the alarm day/date register
    #[must_use]
    pub fn day_date(&self) -> AlarmDayDate {
        self.day_date
    }

    /// Creates an Alarm 2 configuration from existing register values.
    #[must_use]
    pub fn from_registers(
        minutes: AlarmMinutes,
        hours: AlarmHours,
        day_date: AlarmDayDate,
    ) -> Self {
        DS3231Alarm2 {
            minutes,
            hours,
            day_date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alarm1_every_second() {
        let config = Alarm1Config::EverySecond;
        let alarm = DS3231Alarm1::from_config(&config).unwrap();

        assert!(alarm.seconds().alarm_mask1());
        assert!(alarm.minutes().alarm_mask2());
        assert!(alarm.hours().alarm_mask3());
        assert!(alarm.day_date().alarm_mask4());
    }

    #[test]
    fn test_alarm1_at_seconds() {
        let config = Alarm1Config::AtSeconds { seconds: 30 };
        let alarm = DS3231Alarm1::from_config(&config).unwrap();

        assert!(!alarm.seconds().alarm_mask1());
        assert_eq!(alarm.seconds().seconds(), 0);
        assert_eq!(alarm.seconds().ten_seconds(), 3);
        assert!(alarm.minutes().alarm_mask2());
        assert!(alarm.hours().alarm_mask3());
        assert!(alarm.day_date().alarm_mask4());
    }

    #[test]
    fn test_alarm1_at_time_24_hour() {
        let config = Alarm1Config::AtTime {
            hours: 15,
            minutes: 30,
            seconds: 45,
            is_pm: None, // 24-hour mode
        };
        let alarm = DS3231Alarm1::from_config(&config).unwrap();

        assert!(!alarm.seconds().alarm_mask1());
        assert!(!alarm.minutes().alarm_mask2());
        assert!(!alarm.hours().alarm_mask3());
        assert!(alarm.day_date().alarm_mask4());

        assert_eq!(
            alarm.hours().time_representation(),
            TimeRepresentation::TwentyFourHour
        );
    }

    #[test]
    fn test_alarm1_at_time_12_hour() {
        let config = Alarm1Config::AtTime {
            hours: 3,
            minutes: 30,
            seconds: 45,
            is_pm: Some(true),
        };
        let alarm = DS3231Alarm1::from_config(&config).unwrap();

        assert_eq!(
            alarm.hours().time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(alarm.hours().pm_or_twenty_hours(), 1); // PM flag should be set
    }

    #[test]
    fn test_alarm1_at_time_on_day() {
        let config = Alarm1Config::AtTimeOnDay {
            hours: 9,
            minutes: 0,
            seconds: 0,
            day: 2, // Monday
            is_pm: None,
        };
        let alarm = DS3231Alarm1::from_config(&config).unwrap();

        assert!(!alarm.day_date().alarm_mask4());
        assert_eq!(alarm.day_date().day_date_select(), DayDateSelect::Day);
        assert_eq!(alarm.day_date().day_or_date(), 2);
    }

    #[test]
    fn test_alarm1_at_time_on_date() {
        let config = Alarm1Config::AtTimeOnDate {
            hours: 12,
            minutes: 0,
            seconds: 0,
            date: 15,
            is_pm: None,
        };
        let alarm = DS3231Alarm1::from_config(&config).unwrap();

        assert!(!alarm.day_date().alarm_mask4());
        assert_eq!(alarm.day_date().day_date_select(), DayDateSelect::Date);
        assert_eq!(alarm.day_date().day_or_date(), 5); // BCD ones place of 15
        assert_eq!(alarm.day_date().ten_date(), 1); // BCD tens place of 15
    }

    #[test]
    fn test_alarm2_every_minute() {
        let config = Alarm2Config::EveryMinute;
        let alarm = DS3231Alarm2::from_config(&config).unwrap();

        assert!(alarm.minutes().alarm_mask2());
        assert!(alarm.hours().alarm_mask3());
        assert!(alarm.day_date().alarm_mask4());
    }

    #[test]
    fn test_alarm2_at_minutes() {
        let config = Alarm2Config::AtMinutes { minutes: 15 };
        let alarm = DS3231Alarm2::from_config(&config).unwrap();

        assert!(!alarm.minutes().alarm_mask2());
        assert_eq!(alarm.minutes().minutes(), 5);
        assert_eq!(alarm.minutes().ten_minutes(), 1);
        assert!(alarm.hours().alarm_mask3());
        assert!(alarm.day_date().alarm_mask4());
    }

    #[test]
    fn test_alarm2_at_time() {
        let config = Alarm2Config::AtTime {
            hours: 14,
            minutes: 30,
            is_pm: None,
        };
        let alarm = DS3231Alarm2::from_config(&config).unwrap();

        assert!(!alarm.minutes().alarm_mask2());
        assert!(!alarm.hours().alarm_mask3());
        assert!(alarm.day_date().alarm_mask4());
    }

    #[test]
    fn test_validation_errors() {
        // Test invalid seconds
        let config = Alarm1Config::AtSeconds { seconds: 60 };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidTime("seconds must be 0-59"))
        ));

        // Test invalid day of week
        let config = Alarm1Config::AtTimeOnDay {
            hours: 9,
            minutes: 0,
            seconds: 0,
            day: 8,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidDayOfWeek)
        ));

        // Test invalid date of month
        let config = Alarm2Config::AtTimeOnDate {
            hours: 12,
            minutes: 0,
            date: 32,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidDateOfMonth)
        ));

        // Test invalid 12-hour time
        let config = Alarm1Config::AtTime {
            hours: 13,
            minutes: 0,
            seconds: 0,
            is_pm: Some(true),
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidTime(
                "hours must be 1-12 in 12-hour mode"
            ))
        ));

        // Test invalid 24-hour time
        let config = Alarm2Config::AtTime {
            hours: 24,
            minutes: 0,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidTime(
                "hours must be 0-23 in 24-hour mode"
            ))
        ));
    }

    #[test]
    fn test_from_registers() {
        let seconds = AlarmSeconds(0x30);
        let minutes = AlarmMinutes(0x45);
        let hours = AlarmHours(0x12);
        let day_date = AlarmDayDate(0x15);

        let alarm1 = DS3231Alarm1::from_registers(seconds, minutes, hours, day_date);
        assert_eq!(alarm1.seconds(), seconds);
        assert_eq!(alarm1.minutes(), minutes);
        assert_eq!(alarm1.hours(), hours);
        assert_eq!(alarm1.day_date(), day_date);

        let alarm2 = DS3231Alarm2::from_registers(minutes, hours, day_date);
        assert_eq!(alarm2.minutes(), minutes);
        assert_eq!(alarm2.hours(), hours);
        assert_eq!(alarm2.day_date(), day_date);
    }

    #[test]
    fn test_alarm1_at_minutes_seconds() {
        let config = Alarm1Config::AtMinutesSeconds {
            minutes: 15,
            seconds: 30,
        };
        let alarm = DS3231Alarm1::from_config(&config).unwrap();

        assert!(!alarm.seconds().alarm_mask1());
        assert_eq!(alarm.seconds().seconds(), 0);
        assert_eq!(alarm.seconds().ten_seconds(), 3);

        assert!(!alarm.minutes().alarm_mask2());
        assert_eq!(alarm.minutes().minutes(), 5);
        assert_eq!(alarm.minutes().ten_minutes(), 1);

        assert!(alarm.hours().alarm_mask3());
        assert!(alarm.day_date().alarm_mask4());
    }

    #[test]
    fn test_alarm2_at_time_on_date() {
        let config = Alarm2Config::AtTimeOnDate {
            hours: 8,
            minutes: 30,
            date: 15,
            is_pm: None,
        };
        let alarm = DS3231Alarm2::from_config(&config).unwrap();

        assert!(!alarm.minutes().alarm_mask2());
        assert!(!alarm.hours().alarm_mask3());
        assert!(!alarm.day_date().alarm_mask4());
        assert_eq!(alarm.day_date().day_date_select(), DayDateSelect::Date);
        assert_eq!(alarm.day_date().day_or_date(), 5); // BCD ones place of 15
        assert_eq!(alarm.day_date().ten_date(), 1); // BCD tens place of 15
    }

    #[test]
    fn test_alarm2_at_time_on_day() {
        let config = Alarm2Config::AtTimeOnDay {
            hours: 17,
            minutes: 45,
            day: 6, // Friday
            is_pm: None,
        };
        let alarm = DS3231Alarm2::from_config(&config).unwrap();

        assert!(!alarm.minutes().alarm_mask2());
        assert!(!alarm.hours().alarm_mask3());
        assert!(!alarm.day_date().alarm_mask4());
        assert_eq!(alarm.day_date().day_date_select(), DayDateSelect::Day);
        assert_eq!(alarm.day_date().day_or_date(), 6);
    }

    #[test]
    fn test_alarm_error_from_datetime_error() {
        use crate::datetime::DS3231DateTimeError;

        let datetime_error = DS3231DateTimeError::InvalidDateTime;
        let alarm_error = AlarmError::from(datetime_error);
        assert!(matches!(alarm_error, AlarmError::DateTime(_)));
    }

    #[test]
    fn test_alarm_error_debug_formatting() {
        extern crate alloc;

        let invalid_time_error = AlarmError::InvalidTime("test message");
        let debug_str = alloc::format!("{:?}", invalid_time_error);
        assert!(debug_str.contains("InvalidTime"));
        assert!(debug_str.contains("test message"));

        let invalid_day_error = AlarmError::InvalidDayOfWeek;
        let debug_str = alloc::format!("{:?}", invalid_day_error);
        assert!(debug_str.contains("InvalidDayOfWeek"));

        let invalid_date_error = AlarmError::InvalidDateOfMonth;
        let debug_str = alloc::format!("{:?}", invalid_date_error);
        assert!(debug_str.contains("InvalidDateOfMonth"));
    }

    #[test]
    fn test_alarm1_config_clone_and_partialeq() {
        let config1 = Alarm1Config::AtTime {
            hours: 9,
            minutes: 30,
            seconds: 0,
            is_pm: None,
        };
        let config2 = config1.clone();
        assert_eq!(config1, config2);

        let config3 = Alarm1Config::AtTime {
            hours: 10,
            minutes: 30,
            seconds: 0,
            is_pm: None,
        };
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_alarm2_config_clone_and_partialeq() {
        let config1 = Alarm2Config::AtTime {
            hours: 14,
            minutes: 30,
            is_pm: None,
        };
        let config2 = config1.clone();
        assert_eq!(config1, config2);

        let config3 = Alarm2Config::AtTime {
            hours: 15,
            minutes: 30,
            is_pm: None,
        };
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_alarm1_twelve_hour_edge_cases() {
        // Test 12 AM (midnight)
        let config = Alarm1Config::AtTime {
            hours: 12,
            minutes: 0,
            seconds: 0,
            is_pm: Some(false), // 12 AM
        };
        let alarm = DS3231Alarm1::from_config(&config).unwrap();
        assert_eq!(
            alarm.hours().time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(alarm.hours().pm_or_twenty_hours(), 0); // AM

        // Test 12 PM (noon)
        let config = Alarm1Config::AtTime {
            hours: 12,
            minutes: 0,
            seconds: 0,
            is_pm: Some(true), // 12 PM
        };
        let alarm = DS3231Alarm1::from_config(&config).unwrap();
        assert_eq!(
            alarm.hours().time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(alarm.hours().pm_or_twenty_hours(), 1); // PM
    }

    #[test]
    fn test_alarm2_twelve_hour_edge_cases() {
        // Test 1 AM
        let config = Alarm2Config::AtTime {
            hours: 1,
            minutes: 30,
            is_pm: Some(false),
        };
        let alarm = DS3231Alarm2::from_config(&config).unwrap();
        assert_eq!(
            alarm.hours().time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(alarm.hours().pm_or_twenty_hours(), 0); // AM

        // Test 11 PM
        let config = Alarm2Config::AtTime {
            hours: 11,
            minutes: 45,
            is_pm: Some(true),
        };
        let alarm = DS3231Alarm2::from_config(&config).unwrap();
        assert_eq!(
            alarm.hours().time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(alarm.hours().pm_or_twenty_hours(), 1); // PM
    }

    #[test]
    fn test_comprehensive_validation_errors() {
        // Test all Alarm1Config validation errors
        assert!(matches!(
            Alarm1Config::AtSeconds { seconds: 60 }.validate(),
            Err(AlarmError::InvalidTime("seconds must be 0-59"))
        ));

        assert!(matches!(
            Alarm1Config::AtMinutesSeconds {
                minutes: 60,
                seconds: 30
            }
            .validate(),
            Err(AlarmError::InvalidTime("minutes must be 0-59"))
        ));

        assert!(matches!(
            Alarm1Config::AtTime {
                hours: 24,
                minutes: 0,
                seconds: 0,
                is_pm: None
            }
            .validate(),
            Err(AlarmError::InvalidTime(
                "hours must be 0-23 in 24-hour mode"
            ))
        ));

        assert!(matches!(
            Alarm1Config::AtTime {
                hours: 0,
                minutes: 0,
                seconds: 0,
                is_pm: Some(true)
            }
            .validate(),
            Err(AlarmError::InvalidTime(
                "hours must be 1-12 in 12-hour mode"
            ))
        ));

        assert!(matches!(
            Alarm1Config::AtTimeOnDate {
                hours: 12,
                minutes: 0,
                seconds: 0,
                date: 0,
                is_pm: None
            }
            .validate(),
            Err(AlarmError::InvalidDateOfMonth)
        ));

        assert!(matches!(
            Alarm1Config::AtTimeOnDay {
                hours: 12,
                minutes: 0,
                seconds: 0,
                day: 8,
                is_pm: None
            }
            .validate(),
            Err(AlarmError::InvalidDayOfWeek)
        ));

        // Test all Alarm2Config validation errors
        assert!(matches!(
            Alarm2Config::AtMinutes { minutes: 60 }.validate(),
            Err(AlarmError::InvalidTime("minutes must be 0-59"))
        ));

        assert!(matches!(
            Alarm2Config::AtTime {
                hours: 24,
                minutes: 0,
                is_pm: None
            }
            .validate(),
            Err(AlarmError::InvalidTime(
                "hours must be 0-23 in 24-hour mode"
            ))
        ));

        assert!(matches!(
            Alarm2Config::AtTime {
                hours: 13,
                minutes: 0,
                is_pm: Some(true)
            }
            .validate(),
            Err(AlarmError::InvalidTime(
                "hours must be 1-12 in 12-hour mode"
            ))
        ));

        assert!(matches!(
            Alarm2Config::AtTimeOnDate {
                hours: 12,
                minutes: 0,
                date: 32,
                is_pm: None
            }
            .validate(),
            Err(AlarmError::InvalidDateOfMonth)
        ));

        assert!(matches!(
            Alarm2Config::AtTimeOnDay {
                hours: 12,
                minutes: 0,
                day: 0,
                is_pm: None
            }
            .validate(),
            Err(AlarmError::InvalidDayOfWeek)
        ));
    }

    #[test]
    fn test_alarm_register_accessors() {
        let seconds = AlarmSeconds(0x35);
        let minutes = AlarmMinutes(0x42);
        let hours = AlarmHours(0x18);
        let day_date = AlarmDayDate(0x25);

        let alarm1 = DS3231Alarm1::from_registers(seconds, minutes, hours, day_date);
        assert_eq!(alarm1.seconds(), seconds);
        assert_eq!(alarm1.minutes(), minutes);
        assert_eq!(alarm1.hours(), hours);
        assert_eq!(alarm1.day_date(), day_date);

        let alarm2 = DS3231Alarm2::from_registers(minutes, hours, day_date);
        assert_eq!(alarm2.minutes(), minutes);
        assert_eq!(alarm2.hours(), hours);
        assert_eq!(alarm2.day_date(), day_date);
    }

    #[test]
    fn test_ds3231_alarm_copy_clone_partialeq() {
        let seconds = AlarmSeconds(0x30);
        let minutes = AlarmMinutes(0x45);
        let hours = AlarmHours(0x12);
        let day_date = AlarmDayDate(0x15);

        let alarm1 = DS3231Alarm1::from_registers(seconds, minutes, hours, day_date);
        let alarm1_copy = alarm1;
        let alarm1_clone = alarm1.clone();

        assert_eq!(alarm1, alarm1_copy);
        assert_eq!(alarm1, alarm1_clone);

        let alarm2 = DS3231Alarm2::from_registers(minutes, hours, day_date);
        let alarm2_copy = alarm2;
        let alarm2_clone = alarm2.clone();

        assert_eq!(alarm2, alarm2_copy);
        assert_eq!(alarm2, alarm2_clone);
    }

    #[test]
    fn test_alarm_bcd_edge_cases() {
        // Test maximum valid BCD values
        let config = Alarm1Config::AtTime {
            hours: 23,
            minutes: 59,
            seconds: 59,
            is_pm: None,
        };
        let alarm = DS3231Alarm1::from_config(&config).unwrap();
        assert_eq!(alarm.seconds().seconds(), 9);
        assert_eq!(alarm.seconds().ten_seconds(), 5);
        assert_eq!(alarm.minutes().minutes(), 9);
        assert_eq!(alarm.minutes().ten_minutes(), 5);

        // Test minimum valid BCD values
        let config = Alarm2Config::AtTime {
            hours: 0,
            minutes: 0,
            is_pm: None,
        };
        let alarm = DS3231Alarm2::from_config(&config).unwrap();
        assert_eq!(alarm.minutes().minutes(), 0);
        assert_eq!(alarm.minutes().ten_minutes(), 0);
    }

    #[test]
    fn test_alarm_date_edge_cases() {
        // Test date 1
        let config = Alarm1Config::AtTimeOnDate {
            hours: 12,
            minutes: 0,
            seconds: 0,
            date: 1,
            is_pm: None,
        };
        let alarm = DS3231Alarm1::from_config(&config).unwrap();
        assert_eq!(alarm.day_date().day_or_date(), 1);
        assert_eq!(alarm.day_date().ten_date(), 0);

        // Test date 31
        let config = Alarm2Config::AtTimeOnDate {
            hours: 12,
            minutes: 0,
            date: 31,
            is_pm: None,
        };
        let alarm = DS3231Alarm2::from_config(&config).unwrap();
        assert_eq!(alarm.day_date().day_or_date(), 1);
        assert_eq!(alarm.day_date().ten_date(), 3);
    }

    #[test]
    fn test_alarm_day_edge_cases() {
        // Test all valid days (1-7)
        for day in 1..=7 {
            let config = Alarm1Config::AtTimeOnDay {
                hours: 12,
                minutes: 0,
                seconds: 0,
                day,
                is_pm: None,
            };
            let alarm = DS3231Alarm1::from_config(&config).unwrap();
            assert_eq!(alarm.day_date().day_or_date(), day);
            assert_eq!(alarm.day_date().day_date_select(), DayDateSelect::Day);
        }
    }

    #[cfg(feature = "defmt")]
    #[test]
    fn test_alarm_config_defmt_formatting() {
        // Test defmt formatting for alarm configs
        let alarm1_config = Alarm1Config::AtTime {
            hours: 9,
            minutes: 30,
            seconds: 0,
            is_pm: None,
        };
        let _formatted = defmt::Debug2Format(&alarm1_config);

        let alarm2_config = Alarm2Config::EveryMinute;
        let _formatted = defmt::Debug2Format(&alarm2_config);
    }
}
