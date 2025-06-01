# DS3231 Real-Time Clock Driver

[![Crates.io](https://img.shields.io/crates/v/ds3231.svg)](https://crates.io/crates/ds3231)
[![Documentation](https://docs.rs/ds3231/badge.svg)](https://docs.rs/ds3231)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](README.md)
[![Coverage Status](https://coveralls.io/repos/github/liebman/ds3231-rs/badge.svg?branch=main)](https://coveralls.io/github/liebman/ds3231-rs?branch=main)

A platform-agnostic Rust driver for the DS3231 Real-Time Clock, built on the `embedded-hal` ecosystem.
The DS3231 is a low-cost, extremely accurate I²C real-time clock (RTC) with an integrated
temperature-compensated crystal oscillator (TCXO).

- Both blocking and async I²C operation support
- Full register access (time/date, alarms, control, status)
- Optional logging support via `log` or `defmt`
- No `unsafe` code
- Comprehensive error handling

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
ds3231 = "0.1"  # Replace with actual version
```

### Blocking Example

```rust
use ds3231::{DS3231, Config, TimeRepresentation, SquareWaveFrequency, InterruptControl, Ocillator};

// Create configuration
let config = Config {
    time_representation: TimeRepresentation::TwentyFourHour,
    square_wave_frequency: SquareWaveFrequency::Hz1,
    interrupt_control: InterruptControl::SquareWave,
    battery_backed_square_wave: false,
    oscillator_enable: Ocillator::Enabled,
};

// Initialize device with I2C
let mut rtc = DS3231::new(i2c, 0x68);

// Configure the device
rtc.configure(&config)?;

// Get current date/time
let datetime = rtc.datetime()?;
```

### Async Example

Enable the async feature in your `Cargo.toml`:

```toml
[dependencies]
ds3231 = { version = "0.1", features = ["async"] }
```

Then use with async/await:

```rust
use ds3231::asynch::DS3231;

// Initialize device
let mut rtc = DS3231::new(i2c, 0x68);

// Configure asynchronously
rtc.configure(&config).await?;

// Get current date/time asynchronously
let datetime = rtc.datetime().await?;
```

## Features

The crate can be compiled with the following features:

- `async`: Enables async I²C support
- `log`: Enables logging via the `log` crate
- `defmt`: Enables logging via the `defmt` crate
- `temperature_f32` - Enables temperature reading as f32

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
