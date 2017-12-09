//! # Clerk
//!
//! Clerk is a generic and hardware agnostic libary to controll HD44780 compliant LCD displays.

#![no_std]

#[macro_use]
extern crate bitflags;

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
pub use display::{Display, SeekFrom, ShiftTo};
pub use hal::{DataPins4Lines, DataPins8Lines, Delay, Direction, DisplayHardwareLayer, Level,
              ParallelConnection, Pins, Send, Recieve, Init, WriteMode, ReadMode};
