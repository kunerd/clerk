//! # Clerk
//!
//! Clerk is a generic and hardware agnostic libary to controll HD44780 compliant LCD displays.

#[macro_use]
extern crate bitflags;

mod display;
mod entry_mode;
mod display_control;
mod lines;

use entry_mode::EntryModeBuilder;
use display_control::DisplayControlBuilder;
use lines::FIRST_LINE_ADDRESS;

pub use lines::DefaultLines;
pub use display::{Display, DisplayPins, DisplayHardwareLayer, SeekFrom, ShiftTo};
