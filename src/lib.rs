//! # Clerk
//!
//! Clerk is a generic and hardware agnostic libary to controll HD44780 compliant LCD displays.
//! Its main goal is to provide all features defined in the HD44780 spec.

#![no_std]

#[macro_use]
extern crate bitflags;
extern crate embedded_hal;

mod hal;
mod lines;
mod display;
mod function_set;
mod entry_mode;
mod display_control;
mod address;

pub use lines::{DefaultLines, Home};
pub use display_control::{CursorBlinking, CursorState, DisplayControlBuilder, DisplayState};
pub use entry_mode::EntryModeBuilder;
pub use function_set::{FunctionSetBuilder, LineNumber};
pub use display::{DdRamDisplay as Display, SeekCgRamFrom, SeekFrom, SetFrom, ShiftTo};
pub use hal::{Delay, DataPins4Lines, DataPins8Lines, Direction, Init, Level,
              ParallelConnection, Pins, ReadMode, Receive, Send, WriteMode};

pub use embedded_hal::blocking::delay::DelayUs;
