//! # DS3231 RTC Alarm Example for ESP32
//!
//! This example demonstrates how to use the DS3231 Real-Time Clock (RTC) with alarm functionality
//! on an ESP32 microcontroller. The example showcases:
//!
//! ## Features
//! - Setting a known date and time on the DS3231
//! - Configuring Alarm 1 to trigger 1 minute after the set time
//! - Real-time monitoring of the current time every 100ms
//! - Displaying time changes along with alarm status
//! - Monitoring the SQW/INT pin level for interrupt detection
//! - Automatic clearing of alarm flags when triggered
//!
//! ## Hardware Connections
//! - **SDA**: GPIO23 (I2C Data)
//! - **SCL**: GPIO15 (I2C Clock)
//! - **SQW/INT**: GPIO22 (Square Wave/Interrupt pin)
//!
//! ## Operation
//! 1. Initializes the DS3231 in interrupt mode (not square wave mode)
//! 2. Sets the initial time to: 2024-12-20 14:30:00
//! 3. Configures Alarm 1 to trigger at: 14:31:00 (1 minute later)
//! 4. Continuously monitors and displays:
//!    - Current date and time
//!    - Alarm 1 and Alarm 2 status flags
//!    - SQW/INT pin level (HIGH/LOW)
//! 5. When the alarm triggers, it displays a notification and clears the flag
//!
//! ## Expected Output
//! ```
//! Time: 2024-12-20 14:30:45 | Alarm1: clear | Alarm2: clear | SQW/INT: HIGH
//! Time: 2024-12-20 14:31:00 | Alarm1: TRIGGERED | Alarm2: clear | SQW/INT: LOW
//! 🚨 ALARM 1 TRIGGERED! Clearing flag...
//! ```
//!
//! ## Target Hardware
//! - ESP32C6 (can be adapted for other ESP32 variants)
//! - DS3231 RTC module
//!
//! ## Usage
//! Flash this example to your ESP32 and monitor the serial output to see the
//! real-time clock and alarm functionality in action.

#![no_std]
#![no_main]

use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use ds3231::{
    Alarm1Config, Config, InterruptControl, Oscillator, SquareWaveFrequency, TimeRepresentation,
    DS3231,
};
use esp_backtrace as _;
use esp_hal::time::Rate;
use esp_hal::{
    clock::CpuClock,
    gpio::Input,
    i2c::master::{Config as I2cConfig, I2c},
    main,
    time::{Duration, Instant},
};
use log::info;

#[main]
fn main() -> ! {
    // Initialize logger
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    info!("DS3231 Alarm Example Starting...");

    // Configure I2C pins
    let sda = peripherals.GPIO23; // SDA pin
    let scl = peripherals.GPIO15; // SCL pin

    // Configure interrupt pin (SQW/INT)
    let sqw_int_pin = Input::new(peripherals.GPIO22, Default::default());

    // Initialize I2C
    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(100));
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .unwrap()
        .with_sda(sda)
        .with_scl(scl);

    // Initialize DS3231
    let mut rtc = DS3231::new(i2c, 0x68);

    // Configure DS3231 for interrupt mode
    let rtc_config = Config {
        time_representation: TimeRepresentation::TwentyFourHour,
        square_wave_frequency: SquareWaveFrequency::Hz1,
        interrupt_control: InterruptControl::Interrupt, // Enable interrupt mode
        battery_backed_square_wave: false,
        oscillator_enable: Oscillator::Enabled,
    };

    match rtc.configure(&rtc_config) {
        Ok(_) => info!("DS3231 configured successfully"),
        Err(e) => {
            info!("Failed to configure DS3231: {:?}", e);
            panic!("DS3231 configuration failed");
        }
    }

    // Set a known date and time: 2024-12-20 14:30:00
    let initial_datetime = NaiveDate::from_ymd_opt(2024, 12, 20)
        .unwrap()
        .and_hms_opt(14, 30, 0)
        .unwrap();

    match rtc.set_datetime(&initial_datetime) {
        Ok(_) => info!(
            "Initial time set to: {}-{:02}-{:02} {:02}:{:02}:{:02}",
            initial_datetime.year(),
            initial_datetime.month(),
            initial_datetime.day(),
            initial_datetime.hour(),
            initial_datetime.minute(),
            initial_datetime.second()
        ),
        Err(e) => {
            info!("Failed to set initial time: {:?}", e);
            panic!("Failed to set initial time");
        }
    }

    // Set alarm for 1 minute later (14:31:00)
    let alarm_config = Alarm1Config::AtTime {
        hours: 14,
        minutes: 31,
        seconds: 0,
        is_pm: None, // 24-hour mode
    };

    match rtc.set_alarm1(&alarm_config) {
        Ok(_) => info!("Alarm set for 14:31:00"),
        Err(e) => {
            info!("Failed to set alarm: {:?}", e);
            panic!("Failed to set alarm");
        }
    }

    // Clear any existing alarm flags
    match rtc.status() {
        Ok(mut status) => {
            status.set_alarm1_flag(false);
            status.set_alarm2_flag(false);
            match rtc.set_status(status) {
                Ok(_) => info!("Alarm flags cleared"),
                Err(e) => info!("Failed to clear alarm flags: {:?}", e),
            }
        }
        Err(e) => info!("Failed to read status: {:?}", e),
    }

    // Enable Alarm 1 interrupt
    match rtc.control() {
        Ok(mut control) => {
            control.set_alarm1_interrupt_enable(true);
            control.set_alarm2_interrupt_enable(false);
            match rtc.set_control(control) {
                Ok(_) => info!("Alarm 1 interrupt enabled"),
                Err(e) => info!("Failed to enable alarm interrupt: {:?}", e),
            }
        }
        Err(e) => info!("Failed to read control register: {:?}", e),
    }

    info!("Starting time monitoring...");
    info!("Current time will be displayed every 100ms when it changes");
    info!("Alarm status will be shown alongside the time");
    info!("SQW/INT pin level will also be monitored");

    let mut last_datetime: Option<NaiveDateTime> = None;
    let mut last_alarm_status = false;
    let mut last_pin_level = sqw_int_pin.is_high();

    loop {
        let loop_start = Instant::now();

        // Read current time
        match rtc.datetime() {
            Ok(current_time) => {
                // Read alarm status
                let (alarm1_flag, alarm2_flag) = match rtc.status() {
                    Ok(status) => (status.alarm1_flag(), status.alarm2_flag()),
                    Err(_) => (false, false),
                };

                // Check pin level
                let current_pin_level = sqw_int_pin.is_high();

                // Display time if it changed or if alarm status changed or pin level changed
                let time_changed = last_datetime != Some(current_time);
                let alarm_status_changed = last_alarm_status != alarm1_flag;
                let pin_level_changed = last_pin_level != current_pin_level;

                if time_changed || alarm_status_changed || pin_level_changed {
                    info!(
                        "Time: {}-{:02}-{:02} {:02}:{:02}:{:02} | Alarm1: {} | Alarm2: {} | SQW/INT: {}",
                        current_time.year(),
                        current_time.month(),
                        current_time.day(),
                        current_time.hour(),
                        current_time.minute(),
                        current_time.second(),
                        if alarm1_flag { "TRIGGERED" } else { "clear" },
                        if alarm2_flag { "TRIGGERED" } else { "clear" },
                        if current_pin_level { "HIGH" } else { "LOW" }
                    );

                    // If alarm triggered, clear the flag
                    if alarm1_flag && !last_alarm_status {
                        info!("🚨 ALARM 1 TRIGGERED! Clearing flag...");
                        match rtc.status() {
                            Ok(mut status) => {
                                status.set_alarm1_flag(false);
                                match rtc.set_status(status) {
                                    Ok(_) => info!("Alarm 1 flag cleared"),
                                    Err(e) => info!("Failed to clear alarm 1 flag: {:?}", e),
                                }
                            }
                            Err(e) => info!("Failed to read status for clearing: {:?}", e),
                        }
                    }

                    last_datetime = Some(current_time);
                    last_alarm_status = alarm1_flag;
                    last_pin_level = current_pin_level;
                }
            }
            Err(e) => {
                info!("Failed to read time: {:?}", e);
            }
        }

        // Wait for the remainder of 100ms
        while loop_start.elapsed() < Duration::from_millis(100) {
            // Busy wait
        }
    }
}
