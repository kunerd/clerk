//! # Clerk
//!
//! Clerk is a generic and hardware agnostic libary to controll HD44780 compliant LCD displays.

#![no_std]

#[macro_use]
extern crate bitflags;

mod display;
mod entry_mode;
mod display_control;
mod function_set;
mod lines;
mod address;

pub use lines::{DefaultLines, Home};
pub use display_control::{CursorBlinking, CursorState, DisplayControlBuilder, DisplayState};
pub use entry_mode::EntryModeBuilder;
pub use function_set::{FunctionSetBuilder, LineNumber};
pub use display::{Delay, Direction, Display, DisplayHardwareLayer, DisplayPins, Level, SeekFrom,
                  ShiftTo};
