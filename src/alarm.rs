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

    /// Converts the register values back to an `Alarm1Config`.
    ///
    /// # Returns
    ///
    /// The `Alarm1Config` that corresponds to the current register values.
    ///
    /// # Errors
    ///
    /// Returns an error if the register values don't form a valid configuration
    /// or contain invalid BCD values.
    pub fn to_config(&self) -> Result<Alarm1Config, AlarmError> {
        // Check mask bit pattern to determine alarm type
        let mask1 = self.seconds.alarm_mask1();
        let mask2 = self.minutes.alarm_mask2();
        let mask3 = self.hours.alarm_mask3();
        let mask4 = self.day_date.alarm_mask4();

        match (mask1, mask2, mask3, mask4) {
            // All masks set - every second
            (true, true, true, true) => Ok(Alarm1Config::EverySecond),

            // Only seconds mask clear - match seconds
            (false, true, true, true) => {
                let seconds = self.decode_bcd_seconds()?;
                Ok(Alarm1Config::AtSeconds { seconds })
            }

            // Seconds and minutes masks clear - match minutes:seconds
            (false, false, true, true) => {
                let seconds = self.decode_bcd_seconds()?;
                let minutes = self.decode_bcd_minutes()?;
                Ok(Alarm1Config::AtMinutesSeconds { minutes, seconds })
            }

            // Only day/date mask set - match time daily
            (false, false, false, true) => {
                let seconds = self.decode_bcd_seconds()?;
                let minutes = self.decode_bcd_minutes()?;
                let (hours, is_pm) = self.decode_bcd_hours()?;
                Ok(Alarm1Config::AtTime {
                    hours,
                    minutes,
                    seconds,
                    is_pm,
                })
            }

            // No masks set - match specific date/day
            (false, false, false, false) => {
                let seconds = self.decode_bcd_seconds()?;
                let minutes = self.decode_bcd_minutes()?;
                let (hours, is_pm) = self.decode_bcd_hours()?;

                if self.day_date.day_date_select() == DayDateSelect::Day {
                    // Day of week alarm
                    let day = self.day_date.day_or_date();
                    Ok(Alarm1Config::AtTimeOnDay {
                        hours,
                        minutes,
                        seconds,
                        day,
                        is_pm,
                    })
                } else {
                    // Date of month alarm
                    let date = self.decode_bcd_day_date()?;
                    Ok(Alarm1Config::AtTimeOnDate {
                        hours,
                        minutes,
                        seconds,
                        date,
                        is_pm,
                    })
                }
            }

            // Invalid mask combination
            _ => Err(AlarmError::InvalidTime(
                "Invalid alarm mask bit combination",
            )),
        }
    }

    fn decode_bcd_seconds(self) -> Result<u8, AlarmError> {
        let ones = self.seconds.seconds();
        let tens = self.seconds.ten_seconds();
        if ones > 9 || tens > 5 {
            return Err(AlarmError::InvalidTime("Invalid BCD seconds value"));
        }
        Ok(tens * 10 + ones)
    }

    fn decode_bcd_minutes(self) -> Result<u8, AlarmError> {
        let ones = self.minutes.minutes();
        let tens = self.minutes.ten_minutes();
        if ones > 9 || tens > 5 {
            return Err(AlarmError::InvalidTime("Invalid BCD minutes value"));
        }
        Ok(tens * 10 + ones)
    }

    fn decode_bcd_hours(self) -> Result<(u8, Option<bool>), AlarmError> {
        let ones = self.hours.hours();
        let tens = self.hours.ten_hours();

        if ones > 9 || tens > 2 {
            return Err(AlarmError::InvalidTime("Invalid BCD hours value"));
        }

        match self.hours.time_representation() {
            TimeRepresentation::TwentyFourHour => {
                let twenty_hours = self.hours.pm_or_twenty_hours();
                let hours = twenty_hours * 20 + tens * 10 + ones;
                if hours > 23 {
                    return Err(AlarmError::InvalidTime("Invalid 24-hour value"));
                }
                Ok((hours, None))
            }
            TimeRepresentation::TwelveHour => {
                let hours = tens * 10 + ones;
                if hours == 0 || hours > 12 {
                    return Err(AlarmError::InvalidTime("Invalid 12-hour value"));
                }
                let is_pm = self.hours.pm_or_twenty_hours() != 0;
                Ok((hours, Some(is_pm)))
            }
        }
    }

    fn decode_bcd_day_date(self) -> Result<u8, AlarmError> {
        let ones = self.day_date.day_or_date();
        let tens = self.day_date.ten_date();
        if ones > 9 || tens > 3 {
            return Err(AlarmError::InvalidTime("Invalid BCD date value"));
        }
        let date = tens * 10 + ones;
        if date == 0 || date > 31 {
            return Err(AlarmError::InvalidTime("Invalid date value"));
        }
        Ok(date)
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

#[cfg(feature = "defmt")]
impl defmt::Format for DS3231Alarm1 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "DS3231Alarm1 {{ ");
        defmt::write!(f, "seconds: {}, ", self.seconds);
        defmt::write!(f, "minutes: {}, ", self.minutes);
        defmt::write!(f, "hours: {}, ", self.hours);
        defmt::write!(f, "day_date: {} ", self.day_date);
        defmt::write!(f, "}}");
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

    /// Converts the register values back to an `Alarm2Config`.
    ///
    /// # Returns
    ///
    /// The `Alarm2Config` that corresponds to the current register values.
    ///
    /// # Errors
    ///
    /// Returns an error if the register values don't form a valid configuration
    /// or contain invalid BCD values.
    pub fn to_config(&self) -> Result<Alarm2Config, AlarmError> {
        // Check mask bit pattern to determine alarm type
        let mask2 = self.minutes.alarm_mask2();
        let mask3 = self.hours.alarm_mask3();
        let mask4 = self.day_date.alarm_mask4();

        match (mask2, mask3, mask4) {
            // All masks set - every minute
            (true, true, true) => Ok(Alarm2Config::EveryMinute),

            // Only minutes mask clear - match minutes
            (false, true, true) => {
                let minutes = self.decode_bcd_minutes()?;
                Ok(Alarm2Config::AtMinutes { minutes })
            }

            // Only day/date mask set - match time daily
            (false, false, true) => {
                let minutes = self.decode_bcd_minutes()?;
                let (hours, is_pm) = self.decode_bcd_hours()?;
                Ok(Alarm2Config::AtTime {
                    hours,
                    minutes,
                    is_pm,
                })
            }

            // No masks set - match specific date/day
            (false, false, false) => {
                let minutes = self.decode_bcd_minutes()?;
                let (hours, is_pm) = self.decode_bcd_hours()?;

                if self.day_date.day_date_select() == DayDateSelect::Day {
                    // Day of week alarm
                    let day = self.day_date.day_or_date();
                    Ok(Alarm2Config::AtTimeOnDay {
                        hours,
                        minutes,
                        day,
                        is_pm,
                    })
                } else {
                    // Date of month alarm
                    let date = self.decode_bcd_day_date()?;
                    Ok(Alarm2Config::AtTimeOnDate {
                        hours,
                        minutes,
                        date,
                        is_pm,
                    })
                }
            }

            // Invalid mask combination
            _ => Err(AlarmError::InvalidTime(
                "Invalid alarm mask bit combination",
            )),
        }
    }

    fn decode_bcd_minutes(self) -> Result<u8, AlarmError> {
        let ones = self.minutes.minutes();
        let tens = self.minutes.ten_minutes();
        if ones > 9 || tens > 5 {
            return Err(AlarmError::InvalidTime("Invalid BCD minutes value"));
        }
        Ok(tens * 10 + ones)
    }

    fn decode_bcd_hours(self) -> Result<(u8, Option<bool>), AlarmError> {
        let ones = self.hours.hours();
        let tens = self.hours.ten_hours();

        if ones > 9 || tens > 2 {
            return Err(AlarmError::InvalidTime("Invalid BCD hours value"));
        }

        match self.hours.time_representation() {
            TimeRepresentation::TwentyFourHour => {
                let twenty_hours = self.hours.pm_or_twenty_hours();
                let hours = twenty_hours * 20 + tens * 10 + ones;
                if hours > 23 {
                    return Err(AlarmError::InvalidTime("Invalid 24-hour value"));
                }
                Ok((hours, None))
            }
            TimeRepresentation::TwelveHour => {
                let hours = tens * 10 + ones;
                if hours == 0 || hours > 12 {
                    return Err(AlarmError::InvalidTime("Invalid 12-hour value"));
                }
                let is_pm = self.hours.pm_or_twenty_hours() != 0;
                Ok((hours, Some(is_pm)))
            }
        }
    }

    fn decode_bcd_day_date(self) -> Result<u8, AlarmError> {
        let ones = self.day_date.day_or_date();
        let tens = self.day_date.ten_date();
        if ones > 9 || tens > 3 {
            return Err(AlarmError::InvalidTime("Invalid BCD date value"));
        }
        let date = tens * 10 + ones;
        if date == 0 || date > 31 {
            return Err(AlarmError::InvalidTime("Invalid date value"));
        }
        Ok(date)
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

#[cfg(feature = "defmt")]
impl defmt::Format for DS3231Alarm2 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "DS3231Alarm2 {{ ");
        defmt::write!(f, "minutes: {}, ", self.minutes);
        defmt::write!(f, "hours: {}, ", self.hours);
        defmt::write!(f, "day_date: {} ", self.day_date);
        defmt::write!(f, "}}");
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use alloc::vec;

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
    fn test_alarm1_to_config_round_trip() {
        // Test various configurations to ensure round trip conversion works
        let configs = vec![
            Alarm1Config::EverySecond,
            Alarm1Config::AtSeconds { seconds: 30 },
            Alarm1Config::AtMinutesSeconds {
                minutes: 15,
                seconds: 45,
            },
            Alarm1Config::AtTime {
                hours: 9,
                minutes: 30,
                seconds: 0,
                is_pm: None,
            },
            Alarm1Config::AtTimeOnDate {
                hours: 12,
                minutes: 0,
                seconds: 0,
                date: 15,
                is_pm: None,
            },
            Alarm1Config::AtTimeOnDay {
                hours: 18,
                minutes: 45,
                seconds: 30,
                day: 5,
                is_pm: None,
            },
        ];

        for config in configs {
            let alarm = DS3231Alarm1::from_config(&config).unwrap();
            let converted_back = alarm.to_config().unwrap();
            assert_eq!(config, converted_back);
        }
    }

    #[test]
    fn test_alarm2_to_config_round_trip() {
        // Test various configurations to ensure round trip conversion works
        let configs = vec![
            Alarm2Config::EveryMinute,
            Alarm2Config::AtMinutes { minutes: 30 },
            Alarm2Config::AtTime {
                hours: 14,
                minutes: 30,
                is_pm: None,
            },
            Alarm2Config::AtTimeOnDate {
                hours: 8,
                minutes: 15,
                date: 25,
                is_pm: None,
            },
            Alarm2Config::AtTimeOnDay {
                hours: 20,
                minutes: 0,
                day: 3,
                is_pm: None,
            },
        ];

        for config in configs {
            let alarm = DS3231Alarm2::from_config(&config).unwrap();
            let converted_back = alarm.to_config().unwrap();
            assert_eq!(config, converted_back);
        }
    }

    #[test]
    fn test_decode_specific_register_values() {
        // Test the specific register values from the failing test
        let seconds = AlarmSeconds(0x30); // 30 seconds with no mask bit
        let minutes = AlarmMinutes(0x45); // 45 minutes with no mask bit
        let hours = AlarmHours(0x12); // 12 hours with no mask bit
        let day_date = AlarmDayDate(0x15); // 15 date with no mask bit

        let alarm = DS3231Alarm1::from_registers(seconds, minutes, hours, day_date);
        let config = alarm.to_config().unwrap();

        match config {
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
            _ => panic!("Expected AtTimeOnDate configuration, got {:?}", config),
        }
    }

    #[test]
    fn test_decode_specific_alarm2_register_values() {
        // Test the specific register values from the failing alarm2 test
        let minutes = AlarmMinutes(0x45); // 45 minutes with no mask bit
        let hours = AlarmHours(0x12); // 12 hours with no mask bit
        let day_date = AlarmDayDate(0x15); // 15 date with no mask bit

        let alarm = DS3231Alarm2::from_registers(minutes, hours, day_date);
        let config = alarm.to_config().unwrap();

        match config {
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
            _ => panic!("Expected AtTimeOnDate configuration, got {:?}", config),
        }
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
        let hours = AlarmHours(0x12);
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

    #[test]
    fn test_alarm_debug_formatting_coverage() {
        // Test Debug implementation for various alarm configs
        let config1 = Alarm1Config::EverySecond;
        let debug_str = alloc::format!("{:?}", config1);
        assert!(debug_str.contains("EverySecond"));

        let config2 = Alarm1Config::AtTimeOnDate {
            hours: 12,
            minutes: 30,
            seconds: 15,
            date: 25,
            is_pm: Some(true),
        };
        let debug_str = alloc::format!("{:?}", config2);
        assert!(debug_str.contains("AtTimeOnDate"));
        assert!(debug_str.contains("12"));
        assert!(debug_str.contains("30"));
        assert!(debug_str.contains("15"));
        assert!(debug_str.contains("25"));

        let config3 = Alarm2Config::AtTimeOnDay {
            hours: 18,
            minutes: 45,
            day: 3,
            is_pm: None,
        };
        let debug_str = alloc::format!("{:?}", config3);
        assert!(debug_str.contains("AtTimeOnDay"));
        assert!(debug_str.contains("18"));
        assert!(debug_str.contains("45"));
        assert!(debug_str.contains("3"));
    }

    #[test]
    fn test_alarm_register_struct_debug_formatting() {
        // Test Debug implementation for DS3231Alarm1 and DS3231Alarm2 structs
        let alarm1 = DS3231Alarm1::from_config(&Alarm1Config::AtTime {
            hours: 15,
            minutes: 30,
            seconds: 45,
            is_pm: None,
        })
        .unwrap();
        let debug_str = alloc::format!("{:?}", alarm1);
        assert!(debug_str.contains("DS3231Alarm1"));

        let alarm2 = DS3231Alarm2::from_config(&Alarm2Config::AtTime {
            hours: 14,
            minutes: 30,
            is_pm: None,
        })
        .unwrap();
        let debug_str = alloc::format!("{:?}", alarm2);
        assert!(debug_str.contains("DS3231Alarm2"));
    }

    #[test]
    fn test_alarm_error_debug_and_from_implementations() {
        // Test various AlarmError variants
        let error1 = AlarmError::InvalidTime("test error message");
        let debug_str = alloc::format!("{:?}", error1);
        assert!(debug_str.contains("InvalidTime"));
        assert!(debug_str.contains("test error message"));

        let error2 = AlarmError::InvalidDayOfWeek;
        let debug_str = alloc::format!("{:?}", error2);
        assert!(debug_str.contains("InvalidDayOfWeek"));

        let error3 = AlarmError::InvalidDateOfMonth;
        let debug_str = alloc::format!("{:?}", error3);
        assert!(debug_str.contains("InvalidDateOfMonth"));

        // Test From implementation
        use crate::datetime::DS3231DateTimeError;
        let datetime_error = DS3231DateTimeError::InvalidDateTime;
        let alarm_error = AlarmError::from(datetime_error);
        match alarm_error {
            AlarmError::DateTime(_) => {}
            _ => panic!("Expected DateTime variant"),
        }
    }

    #[test]
    fn test_alarm_validation_edge_case_coverage() {
        // Test edge cases for all validation methods

        // Valid edge cases that should pass
        assert!(Alarm1Config::AtSeconds { seconds: 0 }.validate().is_ok());
        assert!(Alarm1Config::AtSeconds { seconds: 59 }.validate().is_ok());
        assert!(Alarm1Config::AtMinutesSeconds {
            minutes: 0,
            seconds: 0
        }
        .validate()
        .is_ok());
        assert!(Alarm1Config::AtMinutesSeconds {
            minutes: 59,
            seconds: 59
        }
        .validate()
        .is_ok());

        // 24-hour mode edge cases
        assert!(Alarm1Config::AtTime {
            hours: 0,
            minutes: 0,
            seconds: 0,
            is_pm: None
        }
        .validate()
        .is_ok());
        assert!(Alarm1Config::AtTime {
            hours: 23,
            minutes: 59,
            seconds: 59,
            is_pm: None
        }
        .validate()
        .is_ok());

        // 12-hour mode edge cases
        assert!(Alarm1Config::AtTime {
            hours: 1,
            minutes: 0,
            seconds: 0,
            is_pm: Some(false)
        }
        .validate()
        .is_ok());
        assert!(Alarm1Config::AtTime {
            hours: 12,
            minutes: 59,
            seconds: 59,
            is_pm: Some(true)
        }
        .validate()
        .is_ok());

        // Date edge cases
        assert!(Alarm1Config::AtTimeOnDate {
            hours: 12,
            minutes: 0,
            seconds: 0,
            date: 1,
            is_pm: None
        }
        .validate()
        .is_ok());
        assert!(Alarm1Config::AtTimeOnDate {
            hours: 12,
            minutes: 0,
            seconds: 0,
            date: 31,
            is_pm: None
        }
        .validate()
        .is_ok());

        // Day edge cases
        assert!(Alarm1Config::AtTimeOnDay {
            hours: 12,
            minutes: 0,
            seconds: 0,
            day: 1,
            is_pm: None
        }
        .validate()
        .is_ok());
        assert!(Alarm1Config::AtTimeOnDay {
            hours: 12,
            minutes: 0,
            seconds: 0,
            day: 7,
            is_pm: None
        }
        .validate()
        .is_ok());

        // Test the same for Alarm2Config
        assert!(Alarm2Config::AtMinutes { minutes: 0 }.validate().is_ok());
        assert!(Alarm2Config::AtMinutes { minutes: 59 }.validate().is_ok());
        assert!(Alarm2Config::AtTime {
            hours: 0,
            minutes: 0,
            is_pm: None
        }
        .validate()
        .is_ok());
        assert!(Alarm2Config::AtTime {
            hours: 23,
            minutes: 59,
            is_pm: None
        }
        .validate()
        .is_ok());
        assert!(Alarm2Config::AtTime {
            hours: 1,
            minutes: 0,
            is_pm: Some(false)
        }
        .validate()
        .is_ok());
        assert!(Alarm2Config::AtTime {
            hours: 12,
            minutes: 59,
            is_pm: Some(true)
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn test_bcd_decoding_helper_coverage() {
        // Test all BCD decoding helper methods with various values
        let alarm1 = DS3231Alarm1::from_config(&Alarm1Config::AtTimeOnDate {
            hours: 23,
            minutes: 59,
            seconds: 45,
            date: 28,
            is_pm: None,
        })
        .unwrap();

        // These should decode successfully
        assert_eq!(alarm1.decode_bcd_seconds().unwrap(), 45);
        assert_eq!(alarm1.decode_bcd_minutes().unwrap(), 59);
        let (hours, is_pm) = alarm1.decode_bcd_hours().unwrap();
        assert_eq!(hours, 23);
        assert_eq!(is_pm, None);
        assert_eq!(alarm1.decode_bcd_day_date().unwrap(), 28);

        // Test 12-hour mode decoding
        let alarm1_12h = DS3231Alarm1::from_config(&Alarm1Config::AtTime {
            hours: 11,
            minutes: 30,
            seconds: 15,
            is_pm: Some(true),
        })
        .unwrap();

        let (hours, is_pm) = alarm1_12h.decode_bcd_hours().unwrap();
        assert_eq!(hours, 11);
        assert_eq!(is_pm, Some(true));

        // Test the same for Alarm2
        let alarm2 = DS3231Alarm2::from_config(&Alarm2Config::AtTimeOnDate {
            hours: 20,
            minutes: 45,
            date: 15,
            is_pm: None,
        })
        .unwrap();

        assert_eq!(alarm2.decode_bcd_minutes().unwrap(), 45);
        let (hours, is_pm) = alarm2.decode_bcd_hours().unwrap();
        assert_eq!(hours, 20);
        assert_eq!(is_pm, None);
        assert_eq!(alarm2.decode_bcd_day_date().unwrap(), 15);
    }

    #[test]
    fn test_all_alarm_configuration_methods() {
        // Test all the private configuration methods by testing their outputs
        let alarm1 = DS3231Alarm1::from_config(&Alarm1Config::EverySecond).unwrap();

        // Test configure_every_second was called correctly
        assert!(alarm1.seconds.alarm_mask1());
        assert!(alarm1.minutes.alarm_mask2());
        assert!(alarm1.hours.alarm_mask3());
        assert!(alarm1.day_date.alarm_mask4());

        // Test other configuration methods through their outputs
        let alarm1 = DS3231Alarm1::from_config(&Alarm1Config::AtSeconds { seconds: 45 }).unwrap();
        assert!(!alarm1.seconds.alarm_mask1());
        assert!(alarm1.minutes.alarm_mask2());
        assert!(alarm1.hours.alarm_mask3());
        assert!(alarm1.day_date.alarm_mask4());
        assert_eq!(alarm1.seconds.seconds(), 5);
        assert_eq!(alarm1.seconds.ten_seconds(), 4);

        let alarm1 = DS3231Alarm1::from_config(&Alarm1Config::AtMinutesSeconds {
            minutes: 30,
            seconds: 15,
        })
        .unwrap();
        assert!(!alarm1.seconds.alarm_mask1());
        assert!(!alarm1.minutes.alarm_mask2());
        assert!(alarm1.hours.alarm_mask3());
        assert!(alarm1.day_date.alarm_mask4());
    }

    #[test]
    fn test_create_alarm_time_components_edge_cases() {
        // Test maximum values
        let (minutes, hours) = create_alarm_time_components(23, 59, None).unwrap();
        assert_eq!(minutes.minutes(), 9);
        assert_eq!(minutes.ten_minutes(), 5);
        assert_eq!(
            hours.time_representation(),
            TimeRepresentation::TwentyFourHour
        );

        // Test 12-hour mode edge cases
        let (_, hours) = create_alarm_time_components(12, 0, Some(true)).unwrap();
        assert_eq!(hours.time_representation(), TimeRepresentation::TwelveHour);
        assert_eq!(hours.pm_or_twenty_hours(), 1); // PM

        let (_, hours) = create_alarm_time_components(12, 0, Some(false)).unwrap();
        assert_eq!(hours.time_representation(), TimeRepresentation::TwelveHour);
        assert_eq!(hours.pm_or_twenty_hours(), 0); // AM
    }

    #[test]
    fn test_create_alarm_day_date_component_edge_cases() {
        // Test day mode
        let day_date = create_alarm_day_date_component(7, true).unwrap(); // Sunday
        assert_eq!(day_date.day_date_select(), DayDateSelect::Day);
        assert_eq!(day_date.day_or_date(), 7);

        // Test date mode with edge cases
        let day_date = create_alarm_day_date_component(1, false).unwrap(); // 1st
        assert_eq!(day_date.day_date_select(), DayDateSelect::Date);
        assert_eq!(day_date.day_or_date(), 1);
        assert_eq!(day_date.ten_date(), 0);

        let day_date = create_alarm_day_date_component(31, false).unwrap(); // 31st
        assert_eq!(day_date.day_date_select(), DayDateSelect::Date);
        assert_eq!(day_date.day_or_date(), 1);
        assert_eq!(day_date.ten_date(), 3);
    }

    #[test]
    fn test_to_config_invalid_mask_combinations() {
        // Test Alarm1 invalid mask combination
        let mut alarm1 = DS3231Alarm1::from_config(&Alarm1Config::EverySecond).unwrap();
        // Create an invalid mask pattern (e.g., only mask2 set)
        alarm1.seconds.set_alarm_mask1(false);
        alarm1.minutes.set_alarm_mask2(true);
        alarm1.hours.set_alarm_mask3(false);
        alarm1.day_date.set_alarm_mask4(false);

        let result = alarm1.to_config();
        assert!(matches!(result, Err(AlarmError::InvalidTime(_))));

        // Test Alarm2 invalid mask combination
        let mut alarm2 = DS3231Alarm2::from_config(&Alarm2Config::EveryMinute).unwrap();
        // Create an invalid mask pattern (e.g., only mask3 set)
        alarm2.minutes.set_alarm_mask2(false);
        alarm2.hours.set_alarm_mask3(true);
        alarm2.day_date.set_alarm_mask4(false);

        let result = alarm2.to_config();
        assert!(matches!(result, Err(AlarmError::InvalidTime(_))));
    }

    #[test]
    fn test_to_config_day_vs_date_selection() {
        // Test day selection vs date selection logic in Alarm1
        let mut alarm1 = DS3231Alarm1::from_config(&Alarm1Config::AtTimeOnDay {
            hours: 12,
            minutes: 30,
            seconds: 0,
            day: 5,
            is_pm: None,
        })
        .unwrap();

        // Convert and verify it's detected as day mode
        let config = alarm1.to_config().unwrap();
        match config {
            Alarm1Config::AtTimeOnDay { day, .. } => assert_eq!(day, 5),
            _ => panic!("Expected AtTimeOnDay"),
        }

        // Change to date mode and verify
        alarm1.day_date.set_day_date_select(DayDateSelect::Date);
        alarm1.day_date.set_ten_date(0);
        alarm1.day_date.set_day_or_date(5); // Date 5
        let config = alarm1.to_config().unwrap();
        match config {
            Alarm1Config::AtTimeOnDate { date, .. } => assert_eq!(date, 5),
            _ => panic!("Expected AtTimeOnDate"),
        }

        // Test the same for Alarm2
        let mut alarm2 = DS3231Alarm2::from_config(&Alarm2Config::AtTimeOnDay {
            hours: 12,
            minutes: 30,
            day: 3,
            is_pm: None,
        })
        .unwrap();

        let config = alarm2.to_config().unwrap();
        match config {
            Alarm2Config::AtTimeOnDay { day, .. } => assert_eq!(day, 3),
            _ => panic!("Expected AtTimeOnDay"),
        }

        // Change to date mode
        alarm2.day_date.set_day_date_select(DayDateSelect::Date);
        alarm2.day_date.set_ten_date(0);
        alarm2.day_date.set_day_or_date(8); // Date 8
        let config = alarm2.to_config().unwrap();
        match config {
            Alarm2Config::AtTimeOnDate { date, .. } => assert_eq!(date, 8),
            _ => panic!("Expected AtTimeOnDate"),
        }
    }

    #[test]
    fn test_12_hour_pm_flag_detection() {
        // Test 12-hour PM flag detection in Alarm1
        let mut alarm1 = DS3231Alarm1::from_config(&Alarm1Config::AtTime {
            hours: 3,
            minutes: 30,
            seconds: 0,
            is_pm: Some(true),
        })
        .unwrap();

        let config = alarm1.to_config().unwrap();
        match config {
            Alarm1Config::AtTime {
                is_pm: Some(true), ..
            } => {}
            _ => panic!("Expected PM flag to be true"),
        }

        // Test AM flag
        alarm1.hours.set_pm_or_twenty_hours(0); // Set AM
        let config = alarm1.to_config().unwrap();
        match config {
            Alarm1Config::AtTime {
                is_pm: Some(false), ..
            } => {}
            _ => panic!("Expected PM flag to be false"),
        }

        // Test the same for Alarm2
        let mut alarm2 = DS3231Alarm2::from_config(&Alarm2Config::AtTime {
            hours: 8,
            minutes: 15,
            is_pm: Some(true),
        })
        .unwrap();

        let config = alarm2.to_config().unwrap();
        match config {
            Alarm2Config::AtTime {
                is_pm: Some(true), ..
            } => {}
            _ => panic!("Expected PM flag to be true"),
        }

        // Test AM flag for Alarm2
        alarm2.hours.set_pm_or_twenty_hours(0); // Set AM
        let config = alarm2.to_config().unwrap();
        match config {
            Alarm2Config::AtTime {
                is_pm: Some(false), ..
            } => {}
            _ => panic!("Expected PM flag to be false"),
        }
    }

    #[test]
    fn test_validation_error_paths() {
        // Test Alarm1Config validation error paths that are not covered

        // AtMinutesSeconds with invalid seconds
        let config = Alarm1Config::AtMinutesSeconds {
            minutes: 30,
            seconds: 60,
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // AtMinutesSeconds with invalid minutes
        let config = Alarm1Config::AtMinutesSeconds {
            minutes: 60,
            seconds: 30,
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // AtTimeOnDate with invalid date (0)
        let config = Alarm1Config::AtTimeOnDate {
            hours: 12,
            minutes: 30,
            seconds: 45,
            date: 0,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidDateOfMonth)
        ));

        // AtTimeOnDate with invalid date (>31)
        let config = Alarm1Config::AtTimeOnDate {
            hours: 12,
            minutes: 30,
            seconds: 45,
            date: 32,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidDateOfMonth)
        ));

        // AtTimeOnDay with invalid day (0)
        let config = Alarm1Config::AtTimeOnDay {
            hours: 12,
            minutes: 30,
            seconds: 45,
            day: 0,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidDayOfWeek)
        ));

        // AtTimeOnDay with invalid day (>7)
        let config = Alarm1Config::AtTimeOnDay {
            hours: 12,
            minutes: 30,
            seconds: 45,
            day: 8,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidDayOfWeek)
        ));

        // Test Alarm2Config validation error paths

        // AtTimeOnDate with invalid date (0)
        let config = Alarm2Config::AtTimeOnDate {
            hours: 12,
            minutes: 30,
            date: 0,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidDateOfMonth)
        ));

        // AtTimeOnDate with invalid date (>31)
        let config = Alarm2Config::AtTimeOnDate {
            hours: 12,
            minutes: 30,
            date: 32,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidDateOfMonth)
        ));

        // AtTimeOnDay with invalid day (0)
        let config = Alarm2Config::AtTimeOnDay {
            hours: 12,
            minutes: 30,
            day: 0,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidDayOfWeek)
        ));

        // AtTimeOnDay with invalid day (>7)
        let config = Alarm2Config::AtTimeOnDay {
            hours: 12,
            minutes: 30,
            day: 8,
            is_pm: None,
        };
        assert!(matches!(
            config.validate(),
            Err(AlarmError::InvalidDayOfWeek)
        ));

        // AtMinutes with invalid minutes
        let config = Alarm2Config::AtMinutes { minutes: 60 };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // Test validate_time error paths for both configs

        // Test Alarm1Config hours validation in 24-hour mode
        let config = Alarm1Config::AtTime {
            hours: 24,
            minutes: 30,
            seconds: 45,
            is_pm: None,
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // Test Alarm1Config hours validation in 12-hour mode (0 hours)
        let config = Alarm1Config::AtTime {
            hours: 0,
            minutes: 30,
            seconds: 45,
            is_pm: Some(true),
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // Test Alarm1Config hours validation in 12-hour mode (>12 hours)
        let config = Alarm1Config::AtTime {
            hours: 13,
            minutes: 30,
            seconds: 45,
            is_pm: Some(true),
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // Test Alarm1Config minutes validation
        let config = Alarm1Config::AtTime {
            hours: 12,
            minutes: 60,
            seconds: 45,
            is_pm: None,
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // Test Alarm1Config seconds validation
        let config = Alarm1Config::AtTime {
            hours: 12,
            minutes: 30,
            seconds: 60,
            is_pm: None,
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // Test Alarm2Config hours validation in 24-hour mode
        let config = Alarm2Config::AtTime {
            hours: 24,
            minutes: 30,
            is_pm: None,
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // Test Alarm2Config hours validation in 12-hour mode (0 hours)
        let config = Alarm2Config::AtTime {
            hours: 0,
            minutes: 30,
            is_pm: Some(true),
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // Test Alarm2Config hours validation in 12-hour mode (>12 hours)
        let config = Alarm2Config::AtTime {
            hours: 13,
            minutes: 30,
            is_pm: Some(true),
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));

        // Test Alarm2Config minutes validation
        let config = Alarm2Config::AtTime {
            hours: 12,
            minutes: 60,
            is_pm: None,
        };
        assert!(matches!(config.validate(), Err(AlarmError::InvalidTime(_))));
    }

    #[test]
    fn test_to_config_with_invalid_bcd_valid_flags() {
        // Goal: Test that to_config fails if BCD is invalid, even if flags are valid.
        // We set up flags for a mode that decodes all fields (e.g., AtTimeOnDate for Alarm1,
        // AtTimeOnDate for Alarm2: all relevant mask bits false, DY/DT=0 for date mode).

        // --- Common valid components (can be overridden for specific tests) ---
        let mut default_seconds = AlarmSeconds::default(); // 0x00
        default_seconds.set_alarm_mask1(false);

        let mut default_minutes = AlarmMinutes::default(); // 0x00
        default_minutes.set_alarm_mask2(false);
        default_minutes.set_minutes(5); // 05 min
        default_minutes.set_ten_minutes(3); // 30 min -> 35 min

        let mut default_hours_24h = AlarmHours::default(); // 0x00
        default_hours_24h.set_alarm_mask3(false);
        default_hours_24h.set_time_representation(TimeRepresentation::TwentyFourHour);
        default_hours_24h.set_hours(2); // 2 hr
        default_hours_24h.set_ten_hours(1); // 10 hr -> 12 hr

        let mut default_hours_12h = AlarmHours::default(); // 0x00
        default_hours_12h.set_alarm_mask3(false);
        default_hours_12h.set_time_representation(TimeRepresentation::TwelveHour);
        default_hours_12h.set_hours(1); // 1 hr
        default_hours_12h.set_ten_hours(0); // 00 hr -> 1 hr
        default_hours_12h.set_pm_or_twenty_hours(1); // PM

        let mut default_day_date = AlarmDayDate::default(); // 0x00
        default_day_date.set_alarm_mask4(false);
        default_day_date.set_day_date_select(DayDateSelect::Date);
        default_day_date.set_day_or_date(5); // 5th
        default_day_date.set_ten_date(1); // 10th -> 15th

        // --- DS3231Alarm1 Tests ---

        // Invalid BCD seconds (ones > 9)
        let alarm1_s1 = DS3231Alarm1::from_registers(
            AlarmSeconds(0x0A),
            default_minutes,
            default_hours_24h,
            default_day_date,
        );
        let res_s1 = alarm1_s1.to_config();
        assert!(
            matches!(
                res_s1,
                Err(AlarmError::InvalidTime("Invalid BCD seconds value"))
            ),
            "Alarm1 SecOnes: {:?}",
            res_s1
        );

        // Invalid BCD seconds (tens > 5)
        let alarm1_s2 = DS3231Alarm1::from_registers(
            AlarmSeconds(0x60),
            default_minutes,
            default_hours_24h,
            default_day_date,
        );
        let res_s2 = alarm1_s2.to_config();
        assert!(
            matches!(
                res_s2,
                Err(AlarmError::InvalidTime("Invalid BCD seconds value"))
            ),
            "Alarm1 SecTens: {:?}",
            res_s2
        );

        // Invalid BCD minutes (ones > 9)
        let alarm1_m1 = DS3231Alarm1::from_registers(
            default_seconds,
            AlarmMinutes(0x0A),
            default_hours_24h,
            default_day_date,
        );
        let res_m1 = alarm1_m1.to_config();
        assert!(
            matches!(
                res_m1,
                Err(AlarmError::InvalidTime("Invalid BCD minutes value"))
            ),
            "Alarm1 MinOnes: {:?}",
            res_m1
        );

        // Invalid BCD minutes (tens > 5)
        let alarm1_m2 = DS3231Alarm1::from_registers(
            default_seconds,
            AlarmMinutes(0x60),
            default_hours_24h,
            default_day_date,
        );
        let res_m2 = alarm1_m2.to_config();
        assert!(
            matches!(
                res_m2,
                Err(AlarmError::InvalidTime("Invalid BCD minutes value"))
            ),
            "Alarm1 MinTens: {:?}",
            res_m2
        );

        // Invalid BCD hours (ones > 9, 24h mode)
        let mut hours_24_invalid_ones = AlarmHours(0x0A); // Raw BCD for 0 tens, 10 ones
        hours_24_invalid_ones.set_alarm_mask3(false);
        hours_24_invalid_ones.set_time_representation(TimeRepresentation::TwentyFourHour);
        let alarm1_h1 = DS3231Alarm1::from_registers(
            default_seconds,
            default_minutes,
            hours_24_invalid_ones,
            default_day_date,
        );
        let res_h1 = alarm1_h1.to_config();
        assert!(
            matches!(
                res_h1,
                Err(AlarmError::InvalidTime("Invalid BCD hours value"))
            ),
            "Alarm1 Hr24Ones: {:?}",
            res_h1
        );

        // Invalid BCD hours (tens > 2, 24h mode e.g. 0x30)
        let mut hours_24_invalid_tens = AlarmHours(0x30); // Raw BCD for 3 tens, 0 ones
        hours_24_invalid_tens.set_alarm_mask3(false);
        hours_24_invalid_tens.set_time_representation(TimeRepresentation::TwentyFourHour);
        let alarm1_h2 = DS3231Alarm1::from_registers(
            default_seconds,
            default_minutes,
            hours_24_invalid_tens,
            default_day_date,
        );
        let res_h2 = alarm1_h2.to_config();
        assert!(
            matches!(
                res_h2,
                Err(AlarmError::InvalidTime("Invalid 24-hour value"))
            ),
            "Alarm1 Hr24Tens: {:?}",
            res_h2
        );

        // Invalid 24-hour value (e.g. 24:00 which is 0x24 BCD - should be caught by value check)
        let mut hours_24_val_error = AlarmHours(0x24);
        hours_24_val_error.set_alarm_mask3(false);
        hours_24_val_error.set_time_representation(TimeRepresentation::TwentyFourHour);
        let alarm1_h3 = DS3231Alarm1::from_registers(
            default_seconds,
            default_minutes,
            hours_24_val_error,
            default_day_date,
        );
        let res_h3 = alarm1_h3.to_config();
        assert!(
            matches!(
                res_h3,
                Err(AlarmError::InvalidTime("Invalid 24-hour value"))
            ),
            "Alarm1 Hr24Val: {:?}",
            res_h3
        );

        // Invalid BCD hours (ones > 9, 12h mode e.g. 0x1A for 1 tens, 10 ones)
        let mut hours_12_invalid_ones = AlarmHours(0x1A);
        hours_12_invalid_ones.set_alarm_mask3(false);
        hours_12_invalid_ones.set_time_representation(TimeRepresentation::TwelveHour);
        hours_12_invalid_ones.set_pm_or_twenty_hours(0); // AM
        let alarm1_h4 = DS3231Alarm1::from_registers(
            default_seconds,
            default_minutes,
            hours_12_invalid_ones,
            default_day_date,
        );
        let res_h4 = alarm1_h4.to_config();
        assert!(
            matches!(
                res_h4,
                Err(AlarmError::InvalidTime("Invalid BCD hours value"))
            ),
            "Alarm1 Hr12Ones: {:?}",
            res_h4
        );

        // Invalid 12-hour value (00:xx AM which is BCD 0x00 with 12h bit)
        let mut hours_12_val_zero = AlarmHours(0x00);
        hours_12_val_zero.set_alarm_mask3(false);
        hours_12_val_zero.set_time_representation(TimeRepresentation::TwelveHour);
        hours_12_val_zero.set_pm_or_twenty_hours(0); // AM
        let alarm1_h5 = DS3231Alarm1::from_registers(
            default_seconds,
            default_minutes,
            hours_12_val_zero,
            default_day_date,
        );
        let res_h5 = alarm1_h5.to_config();
        assert!(
            matches!(
                res_h5,
                Err(AlarmError::InvalidTime("Invalid 12-hour value"))
            ),
            "Alarm1 Hr12ValZero: {:?}",
            res_h5
        );

        // Invalid 12-hour value (13:xx AM which is BCD 0x13 with 12h bit)
        let mut hours_12_val_thirteen = AlarmHours(0x13);
        hours_12_val_thirteen.set_alarm_mask3(false);
        hours_12_val_thirteen.set_time_representation(TimeRepresentation::TwelveHour);
        hours_12_val_thirteen.set_pm_or_twenty_hours(0); // AM
        let alarm1_h6 = DS3231Alarm1::from_registers(
            default_seconds,
            default_minutes,
            hours_12_val_thirteen,
            default_day_date,
        );
        let res_h6 = alarm1_h6.to_config();
        assert!(
            matches!(
                res_h6,
                Err(AlarmError::InvalidTime("Invalid 12-hour value"))
            ),
            "Alarm1 Hr12ValThirteen: {:?}",
            res_h6
        );

        // Invalid BCD day/date (ones > 9)
        let mut daydate_d1 = AlarmDayDate(0x0A);
        daydate_d1.set_alarm_mask4(false);
        daydate_d1.set_day_date_select(DayDateSelect::Date);
        let alarm1_d1 = DS3231Alarm1::from_registers(
            default_seconds,
            default_minutes,
            default_hours_24h,
            daydate_d1,
        );
        let res_d1 = alarm1_d1.to_config();
        assert!(
            matches!(
                res_d1,
                Err(AlarmError::InvalidTime("Invalid BCD date value"))
            ),
            "Alarm1 DateOnes: {:?}",
            res_d1
        );

        // Invalid BCD day/date (tens > 3, e.g. 0x40 for date)
        let mut daydate_d2 = AlarmDayDate(0x40);
        daydate_d2.set_alarm_mask4(false);
        daydate_d2.set_day_date_select(DayDateSelect::Date);
        let alarm1_d2 = DS3231Alarm1::from_registers(
            default_seconds,
            default_minutes,
            default_hours_24h,
            daydate_d2,
        );
        let res_d2 = alarm1_d2.to_config();
        assert!(
            matches!(res_d2, Err(AlarmError::InvalidTime("Invalid date value"))),
            "Alarm1 DateTens (effectively date 0): {:?}",
            res_d2
        );

        // Invalid date value (00)
        let mut daydate_d3 = AlarmDayDate(0x00); // BCD for 0
        daydate_d3.set_alarm_mask4(false);
        daydate_d3.set_day_date_select(DayDateSelect::Date);
        let alarm1_d3 = DS3231Alarm1::from_registers(
            default_seconds,
            default_minutes,
            default_hours_24h,
            daydate_d3,
        );
        let res_d3 = alarm1_d3.to_config();
        assert!(
            matches!(res_d3, Err(AlarmError::InvalidTime("Invalid date value"))),
            "Alarm1 DateValZero: {:?}",
            res_d3
        );

        // Invalid date value (32 -> BCD 0x32)
        let mut daydate_d4 = AlarmDayDate(0x32); // BCD for 32
        daydate_d4.set_alarm_mask4(false);
        daydate_d4.set_day_date_select(DayDateSelect::Date);
        let alarm1_d4 = DS3231Alarm1::from_registers(
            default_seconds,
            default_minutes,
            default_hours_24h,
            daydate_d4,
        );
        let res_d4 = alarm1_d4.to_config();
        assert!(
            matches!(res_d4, Err(AlarmError::InvalidTime("Invalid date value"))),
            "Alarm1 DateVal32: {:?}",
            res_d4
        );

        // --- DS3231Alarm2 Tests ---
        // For Alarm2, seconds are not used/decoded from AlarmSeconds.

        // Invalid BCD minutes (ones > 9)
        let alarm2_m1 =
            DS3231Alarm2::from_registers(AlarmMinutes(0x0A), default_hours_24h, default_day_date);
        let res_a2m1 = alarm2_m1.to_config();
        assert!(
            matches!(
                res_a2m1,
                Err(AlarmError::InvalidTime("Invalid BCD minutes value"))
            ),
            "Alarm2 MinOnes: {:?}",
            res_a2m1
        );

        // Invalid BCD minutes (tens > 5)
        let alarm2_m2 =
            DS3231Alarm2::from_registers(AlarmMinutes(0x60), default_hours_24h, default_day_date);
        let res_a2m2 = alarm2_m2.to_config();
        assert!(
            matches!(
                res_a2m2,
                Err(AlarmError::InvalidTime("Invalid BCD minutes value"))
            ),
            "Alarm2 MinTens: {:?}",
            res_a2m2
        );

        // Invalid BCD hours (ones > 9, 24h mode)
        let mut hours_24_invalid_ones_a2 = AlarmHours(0x0A);
        hours_24_invalid_ones_a2.set_alarm_mask3(false);
        hours_24_invalid_ones_a2.set_time_representation(TimeRepresentation::TwentyFourHour);
        let alarm2_h1 = DS3231Alarm2::from_registers(
            default_minutes,
            hours_24_invalid_ones_a2,
            default_day_date,
        );
        let res_a2h1 = alarm2_h1.to_config();
        assert!(
            matches!(
                res_a2h1,
                Err(AlarmError::InvalidTime("Invalid BCD hours value"))
            ),
            "Alarm2 Hr24Ones: {:?}",
            res_a2h1
        );

        // Invalid BCD hours (tens > 2, 24h mode e.g. 0x30)
        let mut hours_24_invalid_tens_a2 = AlarmHours(0x30);
        hours_24_invalid_tens_a2.set_alarm_mask3(false);
        hours_24_invalid_tens_a2.set_time_representation(TimeRepresentation::TwentyFourHour);
        let alarm2_h2 = DS3231Alarm2::from_registers(
            default_minutes,
            hours_24_invalid_tens_a2,
            default_day_date,
        );
        let res_a2h2 = alarm2_h2.to_config();
        assert!(
            matches!(
                res_a2h2,
                Err(AlarmError::InvalidTime("Invalid 24-hour value"))
            ),
            "Alarm2 Hr24Tens: {:?}",
            res_a2h2
        );

        // Invalid 24-hour value (e.g. 24:00 which is 0x24 BCD)
        let mut hours_24_val_error_a2 = AlarmHours(0x24);
        hours_24_val_error_a2.set_alarm_mask3(false);
        hours_24_val_error_a2.set_time_representation(TimeRepresentation::TwentyFourHour);
        let alarm2_h3 =
            DS3231Alarm2::from_registers(default_minutes, hours_24_val_error_a2, default_day_date);
        let res_a2h3 = alarm2_h3.to_config();
        assert!(
            matches!(
                res_a2h3,
                Err(AlarmError::InvalidTime("Invalid 24-hour value"))
            ),
            "Alarm2 Hr24Val: {:?}",
            res_a2h3
        );

        // Invalid BCD hours (ones > 9, 12h mode e.g. 0x1A)
        let mut hours_12_invalid_ones_a2 = AlarmHours(0x1A);
        hours_12_invalid_ones_a2.set_alarm_mask3(false);
        hours_12_invalid_ones_a2.set_time_representation(TimeRepresentation::TwelveHour);
        hours_12_invalid_ones_a2.set_pm_or_twenty_hours(0); // AM
        let alarm2_h4 = DS3231Alarm2::from_registers(
            default_minutes,
            hours_12_invalid_ones_a2,
            default_day_date,
        );
        let res_a2h4 = alarm2_h4.to_config();
        assert!(
            matches!(
                res_a2h4,
                Err(AlarmError::InvalidTime("Invalid BCD hours value"))
            ),
            "Alarm2 Hr12Ones: {:?}",
            res_a2h4
        );

        // Invalid 12-hour value (00:xx AM BCD 0x00 with 12h bit)
        let mut hours_12_val_zero_a2 = AlarmHours(0x00);
        hours_12_val_zero_a2.set_alarm_mask3(false);
        hours_12_val_zero_a2.set_time_representation(TimeRepresentation::TwelveHour);
        hours_12_val_zero_a2.set_pm_or_twenty_hours(0); // AM
        let alarm2_h5 =
            DS3231Alarm2::from_registers(default_minutes, hours_12_val_zero_a2, default_day_date);
        let res_a2h5 = alarm2_h5.to_config();
        assert!(
            matches!(
                res_a2h5,
                Err(AlarmError::InvalidTime("Invalid 12-hour value"))
            ),
            "Alarm2 Hr12ValZero: {:?}",
            res_a2h5
        );

        // Invalid 12-hour value (13:xx AM BCD 0x13 with 12h bit)
        let mut hours_12_val_thirteen_a2 = AlarmHours(0x13);
        hours_12_val_thirteen_a2.set_alarm_mask3(false);
        hours_12_val_thirteen_a2.set_time_representation(TimeRepresentation::TwelveHour);
        hours_12_val_thirteen_a2.set_pm_or_twenty_hours(0); // AM
        let alarm2_h6 = DS3231Alarm2::from_registers(
            default_minutes,
            hours_12_val_thirteen_a2,
            default_day_date,
        );
        let res_a2h6 = alarm2_h6.to_config();
        assert!(
            matches!(
                res_a2h6,
                Err(AlarmError::InvalidTime("Invalid 12-hour value"))
            ),
            "Alarm2 Hr12ValThirteen: {:?}",
            res_a2h6
        );

        // Invalid BCD day/date (ones > 9)
        let mut daydate_a2d1 = AlarmDayDate(0x0A);
        daydate_a2d1.set_alarm_mask4(false);
        daydate_a2d1.set_day_date_select(DayDateSelect::Date);
        let alarm2_d1 =
            DS3231Alarm2::from_registers(default_minutes, default_hours_24h, daydate_a2d1);
        let res_a2d1 = alarm2_d1.to_config();
        assert!(
            matches!(
                res_a2d1,
                Err(AlarmError::InvalidTime("Invalid BCD date value"))
            ),
            "Alarm2 DateOnes: {:?}",
            res_a2d1
        );

        // Invalid BCD day/date (tens > 3, e.g. 0x40 for date)
        let mut daydate_a2d2 = AlarmDayDate(0x40);
        daydate_a2d2.set_alarm_mask4(false);
        daydate_a2d2.set_day_date_select(DayDateSelect::Date);
        let alarm2_d2 =
            DS3231Alarm2::from_registers(default_minutes, default_hours_24h, daydate_a2d2);
        let res_a2d2 = alarm2_d2.to_config();
        assert!(
            matches!(res_a2d2, Err(AlarmError::InvalidTime("Invalid date value"))),
            "Alarm2 DateTens (effectively date 0): {:?}",
            res_a2d2
        );

        // Invalid date value (00)
        let mut daydate_a2d3 = AlarmDayDate(0x00);
        daydate_a2d3.set_alarm_mask4(false);
        daydate_a2d3.set_day_date_select(DayDateSelect::Date);
        let alarm2_d3 =
            DS3231Alarm2::from_registers(default_minutes, default_hours_24h, daydate_a2d3);
        let res_a2d3 = alarm2_d3.to_config();
        assert!(
            matches!(res_a2d3, Err(AlarmError::InvalidTime("Invalid date value"))),
            "Alarm2 DateValZero: {:?}",
            res_a2d3
        );

        // Invalid date value (32 -> BCD 0x32)
        let mut daydate_a2d4 = AlarmDayDate(0x32);
        daydate_a2d4.set_alarm_mask4(false);
        daydate_a2d4.set_day_date_select(DayDateSelect::Date);
        let alarm2_d4 =
            DS3231Alarm2::from_registers(default_minutes, default_hours_24h, daydate_a2d4);
        let res_a2d4 = alarm2_d4.to_config();
        assert!(
            matches!(res_a2d4, Err(AlarmError::InvalidTime("Invalid date value"))),
            "Alarm2 DateVal32: {:?}",
            res_a2d4
        );
    }

    #[test]
    fn test_create_alarm_time_components_errors() {
        // Invalid hour
        assert!(matches!(
            create_alarm_time_components(24, 0, None),
            Err(AlarmError::DateTime(DS3231DateTimeError::InvalidDateTime))
        ));

        // Invalid minute
        assert!(matches!(
            create_alarm_time_components(0, 60, None),
            Err(AlarmError::DateTime(DS3231DateTimeError::InvalidDateTime))
        ));
    }

    #[test]
    fn test_create_alarm_day_date_component_errors() {
        // Invalid date
        assert!(matches!(
            create_alarm_day_date_component(32, false), // Date 32
            Err(AlarmError::DateTime(DS3231DateTimeError::InvalidDateTime))
        ));
    }
}
