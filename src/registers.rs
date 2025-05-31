//! Register definitions and bitfield structures for the DS3231 RTC.
//!
//! This module contains all register addresses, bitfield definitions, and
//! related types for interacting with the DS3231 Real-Time Clock registers.

use bitfield::bitfield;

/// Register addresses for the DS3231 RTC.
#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RegAddr {
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

/// Day/Date select for alarm registers (DY/DT bit).
///
/// This controls whether the alarm day/date register matches against
/// the day of the week or the date of the month.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DayDateSelect {
    /// Match against date of the month (1-31)
    Date = 0,
    /// Match against day of the week (1-7, where 1=Sunday)
    Day = 1,
}

impl From<u8> for DayDateSelect {
    /// Creates a `DayDateSelect` from a raw register value.
    ///
    /// # Panics
    /// Panics if the value is not 0 or 1.
    fn from(v: u8) -> Self {
        match v {
            0 => DayDateSelect::Date,
            1 => DayDateSelect::Day,
            _ => panic!("Invalid value for DayDateSelect: {}", v),
        }
    }
}

impl From<DayDateSelect> for u8 {
    /// Converts a `DayDateSelect` to its raw register value.
    fn from(v: DayDateSelect) -> Self {
        v as u8
    }
}

// This macro generates the From<u8> and Into<u8> implementations for the
// register type
macro_rules! from_register_u8 {
    ($typ:ty) => {
        impl From<u8> for $typ {
            fn from(v: u8) -> Self {
                paste::paste!([< $typ >](v))
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

#[cfg(feature = "defmt")]
impl defmt::Format for Seconds {
    fn format(&self, f: defmt::Formatter) {
        let seconds = 10 * self.ten_seconds() + self.seconds();
        defmt::write!(f, "Seconds({}s)", seconds);
    }
}

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

#[cfg(feature = "defmt")]
impl defmt::Format for Minutes {
    fn format(&self, f: defmt::Formatter) {
        let minutes = 10 * self.ten_minutes() + self.minutes();
        defmt::write!(f, "Minutes({}m)", minutes);
    }
}

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

#[cfg(feature = "defmt")]
impl defmt::Format for Hours {
    fn format(&self, f: defmt::Formatter) {
        let hours = 10 * self.ten_hours() + self.hours();
        match self.time_representation() {
            TimeRepresentation::TwentyFourHour => {
                let hours = hours + 20 * self.pm_or_twenty_hours();
                defmt::write!(f, "Hours({}h 24h)", hours);
            }
            TimeRepresentation::TwelveHour => {
                let is_pm = self.pm_or_twenty_hours() != 0;
                defmt::write!(f, "Hours({}h {})", hours, if is_pm { "PM" } else { "AM" });
            }
        }
    }
}

bitfield! {
    /// Day of week register (1-7).
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Day(u8);
    impl Debug;
    /// Day of week (1-7)
    pub day, set_day: 2, 0;
}
from_register_u8!(Day);

#[cfg(feature = "defmt")]
impl defmt::Format for Day {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Day({})", self.day());
    }
}

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

#[cfg(feature = "defmt")]
impl defmt::Format for Date {
    fn format(&self, f: defmt::Formatter) {
        let date = 10 * self.ten_date() + self.date();
        defmt::write!(f, "Date({})", date);
    }
}

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

#[cfg(feature = "defmt")]
impl defmt::Format for Month {
    fn format(&self, f: defmt::Formatter) {
        let month = 10 * self.ten_month() + self.month();
        defmt::write!(f, "Month({}", month);
        if self.century() {
            defmt::write!(f, ", century");
        }
        defmt::write!(f, ")");
    }
}

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

#[cfg(feature = "defmt")]
impl defmt::Format for Year {
    fn format(&self, f: defmt::Formatter) {
        let year = 10 * self.ten_year() + self.year();
        defmt::write!(f, "Year({})", year);
    }
}

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

#[cfg(feature = "defmt")]
impl defmt::Format for Status {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Status(");
        let mut first = true;
        if self.oscillator_stop_flag() {
            defmt::write!(f, "OSF");
            first = false;
        }
        if self.enable_32khz_output() {
            if !first {
                defmt::write!(f, ", ");
            }
            defmt::write!(f, "EN32kHz");
            first = false;
        }
        if self.busy() {
            if !first {
                defmt::write!(f, ", ");
            }
            defmt::write!(f, "BSY");
            first = false;
        }
        if self.alarm2_flag() {
            if !first {
                defmt::write!(f, ", ");
            }
            defmt::write!(f, "A2F");
            first = false;
        }
        if self.alarm1_flag() {
            if !first {
                defmt::write!(f, ", ");
            }
            defmt::write!(f, "A1F");
            first = false;
        }
        if first {
            defmt::write!(f, "clear");
        }
        defmt::write!(f, ")");
    }
}

bitfield! {
    /// Aging offset register for oscillator adjustment.
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct AgingOffset(u8);
    impl Debug;
    /// Aging offset value (-128 to +127)
    pub i8, aging_offset, set_aging_offset: 7, 0;
}
from_register_u8!(AgingOffset);

#[cfg(feature = "defmt")]
impl defmt::Format for AgingOffset {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "AgingOffset({})", self.aging_offset());
    }
}

bitfield! {
    /// Temperature register (integer part).
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct Temperature(u8);
    impl Debug;
    /// Temperature value (-128 to +127)
    pub i8, temperature, set_temperature: 7, 0;
}
from_register_u8!(Temperature);

#[cfg(feature = "defmt")]
impl defmt::Format for Temperature {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Temperature({}°C)", self.temperature());
    }
}

bitfield! {
    /// Temperature fraction register (decimal part).
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct TemperatureFraction(u8);
    impl Debug;
    /// Temperature fraction value (0.00 to 0.99)
    pub temperature_fraction, set_temperature_fraction: 7, 6;

}
from_register_u8!(TemperatureFraction);

#[cfg(feature = "defmt")]
impl defmt::Format for TemperatureFraction {
    fn format(&self, f: defmt::Formatter) {
        // Convert the fraction to a decimal value (0.25 increments)
        let fraction = self.temperature_fraction();
        let quarter_degrees = (fraction >> 6) & 0x03;
        match quarter_degrees {
            0 => defmt::write!(f, "TemperatureFraction(0.00°C)"),
            1 => defmt::write!(f, "TemperatureFraction(0.25°C)"),
            2 => defmt::write!(f, "TemperatureFraction(0.50°C)"),
            3 => defmt::write!(f, "TemperatureFraction(0.75°C)"),
            _ => defmt::write!(f, "TemperatureFraction(invalid)"),
        }
    }
}

// Alarm register types with mask bits and special control bits

bitfield! {
    /// Alarm Seconds register with mask bit (only used by Alarm 1).
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct AlarmSeconds(u8);
    impl Debug;
    /// Alarm mask bit 1 (A1M1)
    pub alarm_mask1, set_alarm_mask1: 7;
    /// Tens place of seconds (0-5)
    pub ten_seconds, set_ten_seconds: 6, 4;
    /// Ones place of seconds (0-9)
    pub seconds, set_seconds: 3, 0;
}
from_register_u8!(AlarmSeconds);

#[cfg(feature = "defmt")]
impl defmt::Format for AlarmSeconds {
    fn format(&self, f: defmt::Formatter) {
        let seconds = 10 * self.ten_seconds() + self.seconds();
        defmt::write!(f, "AlarmSeconds({}s", seconds);
        if self.alarm_mask1() {
            defmt::write!(f, ", masked");
        }
        defmt::write!(f, ")");
    }
}

bitfield! {
    /// Alarm Minutes register with mask bit (used by both Alarm 1 and Alarm 2).
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct AlarmMinutes(u8);
    impl Debug;
    /// Alarm mask bit 2 (A1M2/A2M2)
    pub alarm_mask2, set_alarm_mask2: 7;
    /// Tens place of minutes (0-5)
    pub ten_minutes, set_ten_minutes: 6, 4;
    /// Ones place of minutes (0-9)
    pub minutes, set_minutes: 3, 0;
}
from_register_u8!(AlarmMinutes);

#[cfg(feature = "defmt")]
impl defmt::Format for AlarmMinutes {
    fn format(&self, f: defmt::Formatter) {
        let minutes = 10 * self.ten_minutes() + self.minutes();
        defmt::write!(f, "AlarmMinutes({}m", minutes);
        if self.alarm_mask2() {
            defmt::write!(f, ", masked");
        }
        defmt::write!(f, ")");
    }
}

bitfield! {
    /// Alarm Hours register with mask bit and time format control (used by both Alarm 1 and Alarm 2).
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct AlarmHours(u8);
    impl Debug;
    /// Alarm mask bit 3 (A1M3/A2M3)
    pub alarm_mask3, set_alarm_mask3: 7;
    /// Time representation format (12/24 hour)
    pub from into TimeRepresentation, time_representation, set_time_representation: 6, 6;
    /// PM flag (12-hour) or 20-hour bit (24-hour)
    pub pm_or_twenty_hours, set_pm_or_twenty_hours: 5, 5;
    /// Tens place of hours
    pub ten_hours, set_ten_hours: 4, 4;
    /// Ones place of hours
    pub hours, set_hours: 3, 0;
}
from_register_u8!(AlarmHours);

#[cfg(feature = "defmt")]
impl defmt::Format for AlarmHours {
    fn format(&self, f: defmt::Formatter) {
        let hours = 10 * self.ten_hours() + self.hours();
        match self.time_representation() {
            TimeRepresentation::TwentyFourHour => {
                let hours = hours + 20 * self.pm_or_twenty_hours();
                defmt::write!(f, "AlarmHours({}h", hours);
            }
            TimeRepresentation::TwelveHour => {
                let is_pm = self.pm_or_twenty_hours() != 0;
                defmt::write!(
                    f,
                    "AlarmHours({}h {}",
                    hours,
                    if is_pm { "PM" } else { "AM" }
                );
            }
        }
        if self.alarm_mask3() {
            defmt::write!(f, ", masked");
        }
        defmt::write!(f, ")");
    }
}

bitfield! {
    /// Alarm Day/Date register with mask bit and DY/DT control (used by both Alarm 1 and Alarm 2).
    #[derive(Clone, Copy, Default, PartialEq)]
    pub struct AlarmDayDate(u8);
    impl Debug;
    /// Alarm mask bit 4 (A1M4/A2M4)
    pub alarm_mask4, set_alarm_mask4: 7;
    /// Day/Date select (1=day of week, 0=date of month)
    pub from into DayDateSelect, day_date_select, set_day_date_select: 6, 6;
    /// Tens place of date (0-3) when DY/DT=0, or unused when DY/DT=1
    pub ten_date, set_ten_date: 5, 4;
    /// Day of week (1-7) when DY/DT=1, or ones place of date (0-9) when DY/DT=0
    pub day_or_date, set_day_or_date: 3, 0;
}
from_register_u8!(AlarmDayDate);

#[cfg(feature = "defmt")]
impl defmt::Format for AlarmDayDate {
    fn format(&self, f: defmt::Formatter) {
        match self.day_date_select() {
            DayDateSelect::Day => {
                defmt::write!(f, "AlarmDayDate(day {})", self.day_or_date());
            }
            DayDateSelect::Date => {
                let date = 10 * self.ten_date() + self.day_or_date();
                defmt::write!(f, "AlarmDayDate(date {})", date);
            }
        }
        if self.alarm_mask4() {
            defmt::write!(f, ", masked");
        }
        defmt::write!(f, ")");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_date_select_conversions() {
        // Test DayDateSelect conversions
        assert_eq!(DayDateSelect::from(0), DayDateSelect::Date);
        assert_eq!(DayDateSelect::from(1), DayDateSelect::Day);
        assert_eq!(u8::from(DayDateSelect::Date), 0);
        assert_eq!(u8::from(DayDateSelect::Day), 1);
    }

    #[test]
    #[should_panic(expected = "Invalid value for DayDateSelect: 2")]
    fn test_invalid_day_date_select_conversion() {
        let _ = DayDateSelect::from(2);
    }

    #[test]
    fn test_seconds_register_conversions() {
        // Test valid BCD values
        let seconds = Seconds::from(0x59); // 59 seconds
        assert_eq!(seconds.ten_seconds(), 5);
        assert_eq!(seconds.seconds(), 9);
        assert_eq!(u8::from(seconds), 0x59);

        let seconds = Seconds::from(0x00); // 0 seconds
        assert_eq!(seconds.ten_seconds(), 0);
        assert_eq!(seconds.seconds(), 0);
        assert_eq!(u8::from(seconds), 0x00);

        let seconds = Seconds::from(0x30); // 30 seconds
        assert_eq!(seconds.ten_seconds(), 3);
        assert_eq!(seconds.seconds(), 0);
        assert_eq!(u8::from(seconds), 0x30);
    }

    #[test]
    fn test_minutes_register_conversions() {
        // Test valid BCD values
        let minutes = Minutes::from(0x59); // 59 minutes
        assert_eq!(minutes.ten_minutes(), 5);
        assert_eq!(minutes.minutes(), 9);
        assert_eq!(u8::from(minutes), 0x59);

        let minutes = Minutes::from(0x00); // 0 minutes
        assert_eq!(minutes.ten_minutes(), 0);
        assert_eq!(minutes.minutes(), 0);
        assert_eq!(u8::from(minutes), 0x00);

        let minutes = Minutes::from(0x45); // 45 minutes
        assert_eq!(minutes.ten_minutes(), 4);
        assert_eq!(minutes.minutes(), 5);
        assert_eq!(u8::from(minutes), 0x45);
    }

    #[test]
    fn test_hours_register_conversions() {
        // Test 24-hour mode
        let hours = Hours::from(0x23); // 23 hours (11 PM in 24-hour)
        assert_eq!(
            hours.time_representation(),
            TimeRepresentation::TwentyFourHour
        );
        assert_eq!(hours.pm_or_twenty_hours(), 1); // 20-hour bit set
        assert_eq!(hours.ten_hours(), 0);
        assert_eq!(hours.hours(), 3);
        assert_eq!(u8::from(hours), 0x23);

        // Test 12-hour mode PM
        let hours = Hours::from(0x72); // 12 PM in 12-hour mode
        assert_eq!(hours.time_representation(), TimeRepresentation::TwelveHour);
        assert_eq!(hours.pm_or_twenty_hours(), 1); // PM bit set
        assert_eq!(hours.ten_hours(), 1);
        assert_eq!(hours.hours(), 2);
        assert_eq!(u8::from(hours), 0x72);

        // Test 12-hour mode AM
        let hours = Hours::from(0x48); // 8 AM in 12-hour mode
        assert_eq!(hours.time_representation(), TimeRepresentation::TwelveHour);
        assert_eq!(hours.pm_or_twenty_hours(), 0); // AM bit clear
        assert_eq!(hours.ten_hours(), 0);
        assert_eq!(hours.hours(), 8);
        assert_eq!(u8::from(hours), 0x48);
    }

    #[test]
    fn test_day_register_conversions() {
        // Test valid day values (1-7)
        let day = Day::from(0x01); // Sunday
        assert_eq!(day.day(), 1);
        assert_eq!(u8::from(day), 0x01);

        let day = Day::from(0x07); // Saturday
        assert_eq!(day.day(), 7);
        assert_eq!(u8::from(day), 0x07);

        let day = Day::from(0x04); // Wednesday
        assert_eq!(day.day(), 4);
        assert_eq!(u8::from(day), 0x04);
    }

    #[test]
    fn test_date_register_conversions() {
        // Test valid BCD date values
        let date = Date::from(0x31); // 31st
        assert_eq!(date.ten_date(), 3);
        assert_eq!(date.date(), 1);
        assert_eq!(u8::from(date), 0x31);

        let date = Date::from(0x01); // 1st
        assert_eq!(date.ten_date(), 0);
        assert_eq!(date.date(), 1);
        assert_eq!(u8::from(date), 0x01);

        let date = Date::from(0x15); // 15th
        assert_eq!(date.ten_date(), 1);
        assert_eq!(date.date(), 5);
        assert_eq!(u8::from(date), 0x15);
    }

    #[test]
    fn test_month_register_conversions() {
        // Test month without century bit
        let month = Month::from(0x12); // December
        assert_eq!(month.century(), false);
        assert_eq!(month.ten_month(), 1);
        assert_eq!(month.month(), 2);
        assert_eq!(u8::from(month), 0x12);

        // Test month with century bit
        let month = Month::from(0x81); // January with century bit
        assert_eq!(month.century(), true);
        assert_eq!(month.ten_month(), 0);
        assert_eq!(month.month(), 1);
        assert_eq!(u8::from(month), 0x81);

        let month = Month::from(0x06); // June
        assert_eq!(month.century(), false);
        assert_eq!(month.ten_month(), 0);
        assert_eq!(month.month(), 6);
        assert_eq!(u8::from(month), 0x06);
    }

    #[test]
    fn test_year_register_conversions() {
        // Test valid BCD year values
        let year = Year::from(0x99); // 99
        assert_eq!(year.ten_year(), 9);
        assert_eq!(year.year(), 9);
        assert_eq!(u8::from(year), 0x99);

        let year = Year::from(0x00); // 00
        assert_eq!(year.ten_year(), 0);
        assert_eq!(year.year(), 0);
        assert_eq!(u8::from(year), 0x00);

        let year = Year::from(0x24); // 24
        assert_eq!(year.ten_year(), 2);
        assert_eq!(year.year(), 4);
        assert_eq!(u8::from(year), 0x24);
    }

    #[test]
    fn test_control_register_conversions() {
        // Test control register with all bits set
        let control = Control::from(0xFF);
        assert_eq!(control.oscillator_enable(), Ocillator::Disabled);
        assert_eq!(control.battery_backed_square_wave(), true);
        assert_eq!(control.convert_temperature(), true);
        // 0xFF has bits 4:3 = 11 = 0b11 = 3, which maps to Hz8192
        assert_eq!(control.square_wave_frequency(), SquareWaveFrequency::Hz8192);
        assert_eq!(control.interrupt_control(), InterruptControl::Interrupt);
        assert_eq!(control.alarm2_interrupt_enable(), true);
        assert_eq!(control.alarm1_interrupt_enable(), true);
        assert_eq!(u8::from(control), 0xFF);

        // Test control register with no bits set
        let control = Control::from(0x00);
        assert_eq!(control.oscillator_enable(), Ocillator::Enabled);
        assert_eq!(control.battery_backed_square_wave(), false);
        assert_eq!(control.convert_temperature(), false);
        assert_eq!(control.square_wave_frequency(), SquareWaveFrequency::Hz1);
        assert_eq!(control.interrupt_control(), InterruptControl::SquareWave);
        assert_eq!(control.alarm2_interrupt_enable(), false);
        assert_eq!(control.alarm1_interrupt_enable(), false);
        assert_eq!(u8::from(control), 0x00);

        // Test specific bit combinations
        let control = Control::from(0x1C); // 0x1C = 00011100: bits 4:3 = 11 = Hz8192, bit 2 = 1 = Interrupt
        assert_eq!(control.oscillator_enable(), Ocillator::Enabled);
        assert_eq!(control.battery_backed_square_wave(), false);
        assert_eq!(control.convert_temperature(), false);
        assert_eq!(control.square_wave_frequency(), SquareWaveFrequency::Hz8192); // bits 4:3 = 11 = Hz8192
        assert_eq!(control.interrupt_control(), InterruptControl::Interrupt);
        assert_eq!(control.alarm2_interrupt_enable(), false);
        assert_eq!(control.alarm1_interrupt_enable(), false);
        assert_eq!(u8::from(control), 0x1C);
    }

    #[test]
    fn test_status_register_conversions() {
        // Test status register with all flags set
        let status = Status::from(0x8F);
        assert_eq!(status.oscillator_stop_flag(), true);
        assert_eq!(status.enable_32khz_output(), true);
        assert_eq!(status.busy(), true);
        assert_eq!(status.alarm2_flag(), true);
        assert_eq!(status.alarm1_flag(), true);
        assert_eq!(u8::from(status), 0x8F);

        // Test status register with no flags set
        let status = Status::from(0x00);
        assert_eq!(status.oscillator_stop_flag(), false);
        assert_eq!(status.enable_32khz_output(), false);
        assert_eq!(status.busy(), false);
        assert_eq!(status.alarm2_flag(), false);
        assert_eq!(status.alarm1_flag(), false);
        assert_eq!(u8::from(status), 0x00);

        // Test specific flag combinations
        let status = Status::from(0x88); // OSF and EN32kHz set
        assert_eq!(status.oscillator_stop_flag(), true);
        assert_eq!(status.enable_32khz_output(), true);
        assert_eq!(status.busy(), false);
        assert_eq!(status.alarm2_flag(), false);
        assert_eq!(status.alarm1_flag(), false);
        assert_eq!(u8::from(status), 0x88);
    }

    #[test]
    fn test_aging_offset_register_conversions() {
        // Test positive aging offset
        let aging_offset = AgingOffset::from(0x05);
        assert_eq!(aging_offset.aging_offset(), 5);
        assert_eq!(u8::from(aging_offset), 0x05);

        // Test negative aging offset (two's complement)
        let aging_offset = AgingOffset::from(0xF6); // -10 in two's complement
        assert_eq!(aging_offset.aging_offset(), -10);
        assert_eq!(u8::from(aging_offset), 0xF6);

        // Test zero aging offset
        let aging_offset = AgingOffset::from(0x00);
        assert_eq!(aging_offset.aging_offset(), 0);
        assert_eq!(u8::from(aging_offset), 0x00);

        // Test maximum positive value
        let aging_offset = AgingOffset::from(0x7F); // +127
        assert_eq!(aging_offset.aging_offset(), 127);
        assert_eq!(u8::from(aging_offset), 0x7F);

        // Test maximum negative value
        let aging_offset = AgingOffset::from(0x80); // -128
        assert_eq!(aging_offset.aging_offset(), -128);
        assert_eq!(u8::from(aging_offset), 0x80);
    }

    #[test]
    fn test_temperature_register_conversions() {
        // Test positive temperature
        let temperature = Temperature::from(0x19); // +25°C
        assert_eq!(temperature.temperature(), 25);
        assert_eq!(u8::from(temperature), 0x19);

        // Test negative temperature (two's complement)
        let temperature = Temperature::from(0xF6); // -10°C
        assert_eq!(temperature.temperature(), -10);
        assert_eq!(u8::from(temperature), 0xF6);

        // Test zero temperature
        let temperature = Temperature::from(0x00);
        assert_eq!(temperature.temperature(), 0);
        assert_eq!(u8::from(temperature), 0x00);

        // Test maximum positive temperature
        let temperature = Temperature::from(0x7F); // +127°C
        assert_eq!(temperature.temperature(), 127);
        assert_eq!(u8::from(temperature), 0x7F);

        // Test maximum negative temperature
        let temperature = Temperature::from(0x80); // -128°C
        assert_eq!(temperature.temperature(), -128);
        assert_eq!(u8::from(temperature), 0x80);
    }

    #[test]
    fn test_temperature_fraction_register_conversions() {
        // Test different fraction values
        let temp_frac = TemperatureFraction::from(0x00); // 0.00°C (bits 7-6 are 00)
        assert_eq!(temp_frac.temperature_fraction(), 0b00); // Getter returns the 2-bit value
        assert_eq!(u8::from(temp_frac), 0x00);

        let temp_frac = TemperatureFraction::from(0x40); // 0.25°C (bits 7-6 are 01)
        assert_eq!(temp_frac.temperature_fraction(), 0b01); // Getter returns the 2-bit value
        assert_eq!(u8::from(temp_frac), 0x40);

        let temp_frac = TemperatureFraction::from(0x80); // 0.50°C (bits 7-6 are 10)
        assert_eq!(temp_frac.temperature_fraction(), 0b10); // Getter returns the 2-bit value
        assert_eq!(u8::from(temp_frac), 0x80);

        let temp_frac = TemperatureFraction::from(0xC0); // 0.75°C (bits 7-6 are 11)
        assert_eq!(temp_frac.temperature_fraction(), 0b11); // Getter returns the 2-bit value
        assert_eq!(u8::from(temp_frac), 0xC0);

        // Test arbitrary value to ensure only relevant bits are used by getter
        // and other bits are ignored by getter but preserved by From<u8>/Into<u8>
        let temp_frac_arbitrary = TemperatureFraction::from(0x55); // 0b01010101, bits 7-6 are 01
        assert_eq!(temp_frac_arbitrary.temperature_fraction(), 0b01); // Should be 1 (0.25°C)
        assert_eq!(u8::from(temp_frac_arbitrary), 0x55); // Raw byte preserved

        // Test setting the fraction and converting back to u8
        let mut temp_frac_setter = TemperatureFraction::default(); // Starts at 0x00
        temp_frac_setter.set_temperature_fraction(0b10); // Set fraction to 0.50°C (binary 10)
                                                         // The setter places 0b10 into bits 7-6. So the raw byte should be 0b10000000 = 0x80
        assert_eq!(u8::from(temp_frac_setter), 0x80);
        assert_eq!(temp_frac_setter.temperature_fraction(), 0b10);

        temp_frac_setter.set_temperature_fraction(0b01); // Set fraction to 0.25°C (binary 01)
                                                         // Raw byte should be 0b01000000 = 0x40
        assert_eq!(u8::from(temp_frac_setter), 0x40);
        assert_eq!(temp_frac_setter.temperature_fraction(), 0b01);
    }

    #[test]
    fn test_alarm_seconds_register_conversions() {
        // Test with mask bit set
        let alarm_seconds = AlarmSeconds::from(0x80); // Mask bit set, 0 seconds
        assert_eq!(alarm_seconds.alarm_mask1(), true);
        assert_eq!(alarm_seconds.ten_seconds(), 0);
        assert_eq!(alarm_seconds.seconds(), 0);
        assert_eq!(u8::from(alarm_seconds), 0x80);

        // Test with mask bit clear and valid BCD seconds
        let alarm_seconds = AlarmSeconds::from(0x35); // No mask, 35 seconds
        assert_eq!(alarm_seconds.alarm_mask1(), false);
        assert_eq!(alarm_seconds.ten_seconds(), 3);
        assert_eq!(alarm_seconds.seconds(), 5);
        assert_eq!(u8::from(alarm_seconds), 0x35);

        // Test with mask bit set and BCD seconds
        let alarm_seconds = AlarmSeconds::from(0xB9); // Mask bit set, 39 seconds
        assert_eq!(alarm_seconds.alarm_mask1(), true);
        assert_eq!(alarm_seconds.ten_seconds(), 3); // bits 6:4 = 011 = 3
        assert_eq!(alarm_seconds.seconds(), 9); // bits 3:0 = 1001 = 9
        assert_eq!(u8::from(alarm_seconds), 0xB9);
    }

    #[test]
    fn test_alarm_minutes_register_conversions() {
        // Test with mask bit set
        let alarm_minutes = AlarmMinutes::from(0x80); // Mask bit set, 0 minutes
        assert_eq!(alarm_minutes.alarm_mask2(), true);
        assert_eq!(alarm_minutes.ten_minutes(), 0);
        assert_eq!(alarm_minutes.minutes(), 0);
        assert_eq!(u8::from(alarm_minutes), 0x80);

        // Test with mask bit clear and valid BCD minutes
        let alarm_minutes = AlarmMinutes::from(0x42); // No mask, 42 minutes
        assert_eq!(alarm_minutes.alarm_mask2(), false);
        assert_eq!(alarm_minutes.ten_minutes(), 4);
        assert_eq!(alarm_minutes.minutes(), 2);
        assert_eq!(u8::from(alarm_minutes), 0x42);

        // Test with mask bit set and BCD minutes
        let alarm_minutes = AlarmMinutes::from(0xD7); // Mask bit set, 57 minutes
        assert_eq!(alarm_minutes.alarm_mask2(), true);
        assert_eq!(alarm_minutes.ten_minutes(), 5);
        assert_eq!(alarm_minutes.minutes(), 7);
        assert_eq!(u8::from(alarm_minutes), 0xD7);
    }

    #[test]
    fn test_alarm_hours_register_conversions() {
        // Test 24-hour mode with mask bit
        let alarm_hours = AlarmHours::from(0x95); // Mask bit set, 24-hour, 15 hours
        assert_eq!(alarm_hours.alarm_mask3(), true);
        assert_eq!(
            alarm_hours.time_representation(),
            TimeRepresentation::TwentyFourHour
        );
        assert_eq!(alarm_hours.pm_or_twenty_hours(), 0);
        assert_eq!(alarm_hours.ten_hours(), 1);
        assert_eq!(alarm_hours.hours(), 5);
        assert_eq!(u8::from(alarm_hours), 0x95);

        // Test 12-hour mode PM with mask bit clear
        let alarm_hours = AlarmHours::from(0x72); // No mask, 12-hour PM, 12 hours
        assert_eq!(alarm_hours.alarm_mask3(), false);
        assert_eq!(
            alarm_hours.time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(alarm_hours.pm_or_twenty_hours(), 1); // PM bit
        assert_eq!(alarm_hours.ten_hours(), 1);
        assert_eq!(alarm_hours.hours(), 2);
        assert_eq!(u8::from(alarm_hours), 0x72);

        // Test 12-hour mode AM with mask bit set
        let alarm_hours = AlarmHours::from(0xC8); // Mask bit set, 12-hour AM, 8 hours
        assert_eq!(alarm_hours.alarm_mask3(), true);
        assert_eq!(
            alarm_hours.time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(alarm_hours.pm_or_twenty_hours(), 0); // AM bit
        assert_eq!(alarm_hours.ten_hours(), 0);
        assert_eq!(alarm_hours.hours(), 8);
        assert_eq!(u8::from(alarm_hours), 0xC8);
    }

    #[test]
    fn test_alarm_day_date_register_conversions() {
        // Test day mode with mask bit set
        let alarm_day_date = AlarmDayDate::from(0xC3); // Mask bit set, day mode, day 3
        assert_eq!(alarm_day_date.alarm_mask4(), true);
        assert_eq!(alarm_day_date.day_date_select(), DayDateSelect::Day);
        assert_eq!(alarm_day_date.ten_date(), 0); // Unused in day mode
        assert_eq!(alarm_day_date.day_or_date(), 3);
        assert_eq!(u8::from(alarm_day_date), 0xC3);

        // Test date mode with mask bit clear
        let alarm_day_date = AlarmDayDate::from(0x15); // No mask, date mode, date 15
        assert_eq!(alarm_day_date.alarm_mask4(), false);
        assert_eq!(alarm_day_date.day_date_select(), DayDateSelect::Date);
        assert_eq!(alarm_day_date.ten_date(), 1);
        assert_eq!(alarm_day_date.day_or_date(), 5);
        assert_eq!(u8::from(alarm_day_date), 0x15);

        // Test day mode with mask bit clear
        let alarm_day_date = AlarmDayDate::from(0x47); // No mask, day mode, day 7
        assert_eq!(alarm_day_date.alarm_mask4(), false);
        assert_eq!(alarm_day_date.day_date_select(), DayDateSelect::Day);
        assert_eq!(alarm_day_date.ten_date(), 0); // Unused in day mode
        assert_eq!(alarm_day_date.day_or_date(), 7);
        assert_eq!(u8::from(alarm_day_date), 0x47);

        // Test date mode with mask bit set
        let alarm_day_date = AlarmDayDate::from(0xA9); // Mask bit set, date mode, date 29
        assert_eq!(alarm_day_date.alarm_mask4(), true);
        assert_eq!(alarm_day_date.day_date_select(), DayDateSelect::Date);
        assert_eq!(alarm_day_date.ten_date(), 2);
        assert_eq!(alarm_day_date.day_or_date(), 9);
        assert_eq!(u8::from(alarm_day_date), 0xA9);
    }

    #[test]
    fn test_register_roundtrip_conversions() {
        // Test that all register types can roundtrip through u8 conversion
        let test_values = [
            0x00, 0x55, 0xAA, 0xFF, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE,
        ];

        for &value in &test_values {
            // Test all register types
            assert_eq!(u8::from(Seconds::from(value)), value);
            assert_eq!(u8::from(Minutes::from(value)), value);
            assert_eq!(u8::from(Hours::from(value)), value);
            assert_eq!(u8::from(Day::from(value)), value);
            assert_eq!(u8::from(Date::from(value)), value);
            assert_eq!(u8::from(Month::from(value)), value);
            assert_eq!(u8::from(Year::from(value)), value);
            assert_eq!(u8::from(Control::from(value)), value);
            assert_eq!(u8::from(Status::from(value)), value);
            assert_eq!(u8::from(AgingOffset::from(value)), value);
            assert_eq!(u8::from(Temperature::from(value)), value);
            assert_eq!(u8::from(TemperatureFraction::from(value)), value);
            assert_eq!(u8::from(AlarmSeconds::from(value)), value);
            assert_eq!(u8::from(AlarmMinutes::from(value)), value);
            assert_eq!(u8::from(AlarmHours::from(value)), value);
            assert_eq!(u8::from(AlarmDayDate::from(value)), value);
        }
    }

    #[test]
    fn test_register_bitfield_operations() {
        // Test Seconds register
        let mut seconds = Seconds::default();
        seconds.set_seconds(5);
        seconds.set_ten_seconds(3);
        assert_eq!(seconds.seconds(), 5);
        assert_eq!(seconds.ten_seconds(), 3);

        // Test Minutes register
        let mut minutes = Minutes::default();
        minutes.set_minutes(8);
        minutes.set_ten_minutes(4);
        assert_eq!(minutes.minutes(), 8);
        assert_eq!(minutes.ten_minutes(), 4);

        // Test Hours register
        let mut hours = Hours::default();
        hours.set_time_representation(TimeRepresentation::TwelveHour);
        hours.set_pm_or_twenty_hours(1);
        hours.set_ten_hours(1);
        hours.set_hours(2);
        assert_eq!(hours.time_representation(), TimeRepresentation::TwelveHour);
        assert_eq!(hours.pm_or_twenty_hours(), 1);
        assert_eq!(hours.ten_hours(), 1);
        assert_eq!(hours.hours(), 2);

        // Test Day register
        let mut day = Day::default();
        day.set_day(3);
        assert_eq!(day.day(), 3);

        // Test Date register
        let mut date = Date::default();
        date.set_date(5);
        date.set_ten_date(2);
        assert_eq!(date.date(), 5);
        assert_eq!(date.ten_date(), 2);

        // Test Month register
        let mut month = Month::default();
        month.set_month(2);
        month.set_ten_month(1);
        month.set_century(true);
        assert_eq!(month.month(), 2);
        assert_eq!(month.ten_month(), 1);
        assert!(month.century());

        // Test Year register
        let mut year = Year::default();
        year.set_year(4);
        year.set_ten_year(2);
        assert_eq!(year.year(), 4);
        assert_eq!(year.ten_year(), 2);

        // Test Control register
        let mut control = Control::default();
        control.set_oscillator_enable(Ocillator::Disabled);
        control.set_battery_backed_square_wave(true);
        control.set_convert_temperature(true);
        control.set_square_wave_frequency(SquareWaveFrequency::Hz4096);
        control.set_interrupt_control(InterruptControl::Interrupt);
        control.set_alarm2_interrupt_enable(true);
        control.set_alarm1_interrupt_enable(true);

        assert_eq!(control.oscillator_enable(), Ocillator::Disabled);
        assert!(control.battery_backed_square_wave());
        assert!(control.convert_temperature());
        assert_eq!(control.square_wave_frequency(), SquareWaveFrequency::Hz4096);
        assert_eq!(control.interrupt_control(), InterruptControl::Interrupt);
        assert!(control.alarm2_interrupt_enable());
        assert!(control.alarm1_interrupt_enable());

        // Test Status register
        let mut status = Status::default();
        status.set_oscillator_stop_flag(true);
        status.set_enable_32khz_output(true);
        status.set_busy(true);
        status.set_alarm2_flag(true);
        status.set_alarm1_flag(true);

        assert!(status.oscillator_stop_flag());
        assert!(status.enable_32khz_output());
        assert!(status.busy());
        assert!(status.alarm2_flag());
        assert!(status.alarm1_flag());

        // Test AgingOffset register
        let mut aging_offset = AgingOffset::default();
        aging_offset.set_aging_offset(-10);
        assert_eq!(aging_offset.aging_offset(), -10);

        // Test Temperature register
        let mut temperature = Temperature::default();
        temperature.set_temperature(25);
        assert_eq!(temperature.temperature(), 25);

        // Test TemperatureFraction register
        let mut temp_frac = TemperatureFraction::default(); // default is 0x00
                                                            // The setter `set_temperature_fraction` expects the 2-bit value (0, 1, 2, or 3).
                                                            // To set 0.25°C (which is bits 7-6 = 01), we pass 0b01 to the setter.
        temp_frac.set_temperature_fraction(0b01);
        // The getter `temperature_fraction()` should then return this 2-bit value (0b01).
        assert_eq!(temp_frac.temperature_fraction(), 0b01);
        // The raw u8 value of temp_frac should be 0b01000000 = 0x40, because set_temperature_fraction(0b01)
        // places 01 into bits 7-6.
        assert_eq!(u8::from(temp_frac), 0x40);

        // Test setting another value, e.g., 0.75°C (bits 7-6 = 11)
        temp_frac.set_temperature_fraction(0b11);
        assert_eq!(temp_frac.temperature_fraction(), 0b11); // Getter returns 3
        assert_eq!(u8::from(temp_frac), 0xC0); // Raw u8 should be 0b11000000
    }

    #[test]
    fn test_alarm_register_bitfield_operations() {
        // Test AlarmSeconds register
        let mut alarm_seconds = AlarmSeconds::default();
        alarm_seconds.set_alarm_mask1(true);
        alarm_seconds.set_ten_seconds(3);
        alarm_seconds.set_seconds(5);
        assert!(alarm_seconds.alarm_mask1());
        assert_eq!(alarm_seconds.ten_seconds(), 3);
        assert_eq!(alarm_seconds.seconds(), 5);

        // Test AlarmMinutes register
        let mut alarm_minutes = AlarmMinutes::default();
        alarm_minutes.set_alarm_mask2(true);
        alarm_minutes.set_ten_minutes(4);
        alarm_minutes.set_minutes(8);
        assert!(alarm_minutes.alarm_mask2());
        assert_eq!(alarm_minutes.ten_minutes(), 4);
        assert_eq!(alarm_minutes.minutes(), 8);

        // Test AlarmHours register
        let mut alarm_hours = AlarmHours::default();
        alarm_hours.set_alarm_mask3(true);
        alarm_hours.set_time_representation(TimeRepresentation::TwelveHour);
        alarm_hours.set_pm_or_twenty_hours(1);
        alarm_hours.set_ten_hours(1);
        alarm_hours.set_hours(2);
        assert!(alarm_hours.alarm_mask3());
        assert_eq!(
            alarm_hours.time_representation(),
            TimeRepresentation::TwelveHour
        );
        assert_eq!(alarm_hours.pm_or_twenty_hours(), 1);
        assert_eq!(alarm_hours.ten_hours(), 1);
        assert_eq!(alarm_hours.hours(), 2);

        // Test AlarmDayDate register
        let mut alarm_day_date = AlarmDayDate::default();
        alarm_day_date.set_alarm_mask4(true);
        alarm_day_date.set_day_date_select(DayDateSelect::Day);
        alarm_day_date.set_ten_date(2);
        alarm_day_date.set_day_or_date(5);
        assert!(alarm_day_date.alarm_mask4());
        assert_eq!(alarm_day_date.day_date_select(), DayDateSelect::Day);
        assert_eq!(alarm_day_date.ten_date(), 2);
        assert_eq!(alarm_day_date.day_or_date(), 5);
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
}
