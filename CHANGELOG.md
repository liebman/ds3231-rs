# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

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
[Unreleased]: https://github.com/user/ds3231-rs/compare/v0.1.0...HEAD
