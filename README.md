# clerk

[![Build Status](https://travis-ci.org/kunerd/clerk.svg?branch=master)](https://travis-ci.org/kunerd/clerk)

Hardware independent HD44780 LCD library written in Rust

## Current state
This library is in an very early implementation state, so it is likely that
interfaces will change. Also there is no error handling currently.

- [x] Clear display
- [ ] Return home (but is possible via `seek()`)
- [x] Entry mode settings
- [x] Cursor and display shift
- [ ] Function set
- [x] Display control settings
- [x] Set DDRAM address (high level interface via `seek()`)
- [x] Set CGRAM address
- [x] Read/write DDRAM
- [x] Read/write CGRAM (create custom characters)
- [x] Read busy flag and cursor address

## Documentation
https://docs.rs/clerk
