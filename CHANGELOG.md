# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.3.0] - 2025-06-09

### Breaking Changes

- The public enum `Ocillator` was renamed to `Oscillator` (#10)

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.2.0] - 2025-06-01

### Added

- New `temperature_f32` feature to read temperature as an `f32` value.

### Changed

- Use maybe-async-cfg to support async vs the default sync operation
- Improved alarm support (#4)
- temperature_fraction() now just returns the 2 active bits.

### Deprecated

### Removed

### Fixed

- Don't assume 24hr time representation (#7)

### Security

## [0.1.0]

### Added

- Initial release of the DS3231 RTC driver

### Changed

### Deprecated

### Removed

### Fixed

### Security

<!-- next-url -->
[Unreleased]: https://github.com/user/ds3231-rs/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/liebman/ds3231-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/liebman/ds3231-rs/compare/v0.1.0...v0.2.0
