# clerk

[![Build Status](https://travis-ci.org/kunerd/clerk.svg?branch=master)](https://travis-ci.org/kunerd/clerk)

Hardware independent HD44780 LCD library written in Rust

The library's goal is to provide a high level interface to control HD44780 compliant LCD displays. It does not rely on `std` and therefore it should work on PCs as well as on embedded devices. Its main goal is to provide all features defined in the HD44780 spec.

## Current state
This library is actively maintained and most of the features described in the HD44780 spec are implemented. The current work mainly concentrates on providing a first stable version.

### Features
- [x] Clear display
- [ ] Return home (but is possible via `seek()`)
- [x] Entry mode settings
- [x] Cursor and display shift
- [x] Function set
- [x] Display control settings
- [x] Set DDRAM address (high level interface via `seek()`)
- [x] Set CGRAM address
- [x] Read/write DDRAM
- [x] Read/write CGRAM (create custom characters)
- [x] Read busy flag and cursor address

### TODOs
- more unit and integration testing
- error handling
- feature flags to allow additional (high level) functions
- conditional compilation for different hardware variants (read-only, read-write)
- test on different targets (currently only tested on Raspberry Pi)

### Documentation
https://docs.rs/clerk

## Getting started/help
Have a look at the [How-to](https://github.com/kunerd/clerk/wiki/How-to-use-HD44780-LCD-from-Rust) for a detailed description on getting started with `clerk` on a RaspberryPi.

If you have any questions, just [create a ticket](https://github.com/kunerd/clerk/issues/new) or ping me on Mozillas IRC channels `#rust` or `rust-embedded`.

## Contribution
All kinds of contributions are highly welcome (see TODOs). [Create tickets](https://github.com/kunerd/clerk/issues/new) with feature requests, design ideas and so on. You can also find me on Mozillas IRC channel `#rust` and `#rust-embedded`.

## License
This project is licensed under MIT license ([LICENSE](docs/CONTRIBUTING.md) or https://opensource.org/licenses/MIT)
