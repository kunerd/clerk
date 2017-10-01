//! # Clerk
//!
//! Clerk is a generic and hardware agnostic libary to controll HD44780 compliant LCD displays.

#![no_std]

#[macro_use]
extern crate bitflags;

mod display;
mod entry_mode;
mod display_control;
mod lines;

use lines::FIRST_LINE_ADDRESS;

pub use lines::DefaultLines;
pub use display_control::DisplayControlBuilder;
pub use entry_mode::EntryModeBuilder;
pub use display_control::{DisplayState, CursorState, CursorBlinking};
pub use display::{Display, DisplayPins, Direction, DisplayHardwareLayer, SeekFrom, ShiftTo};
