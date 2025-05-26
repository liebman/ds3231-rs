#![no_std]
#![no_main]

use ds3231::DS3231;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    i2c::master::{Config as I2cConfig, I2c},
    main,
    time::{Duration, Instant, Rate},
};
use log::info;

#[main]
fn main() -> ! {
    // Initialize logger
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    info!("DS3231 Temperature Example Starting...");

    // Configure I2C pins
    let sda = peripherals.GPIO23; // SDA pin
    let scl = peripherals.GPIO15; // SCL pin

    // Initialize I2C
    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(100));
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .unwrap()
        .with_sda(sda)
        .with_scl(scl);

    // Initialize DS3231
    let mut rtc = DS3231::new(i2c, 0x68);

    info!("Starting temperature monitoring...");
    info!("Current temperature will be displayed every minute");

    loop {
        let loop_start = Instant::now();
        // Read temperature
        match rtc.temperature_f32() {
            Ok(temp) => {
                info!("Temperature: {:.2}°C", temp);
            }
            Err(e) => {
                info!("Failed to read temperature: {:?}", e);
            }
        }

        // Wait for the remainder of 1 second
        while loop_start.elapsed() < Duration::from_secs(60) {
            // Busy wait
        }
    }
}
