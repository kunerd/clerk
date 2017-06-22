//! # Clerk
//!
//! Clerk is a generic and hardware agnostic libary to controll HD44780 compliant LCD displays.

#[macro_use]
extern crate bitflags;

use std::thread::sleep;
use std::time::Duration;
use std::iter::Iterator;

// TODO make configurable
// TODO add optional implementation using the busy flag
static E_DELAY: u32 = 5;
const LCD_WIDTH: usize = 16;
const FIRST_LINE_ADDRESS: u8 = 0x00;
const SECOND_LINE_ADDRESS: u8 = 0x40;

bitflags! {
    struct Instructions: u8 {
        const CLEAR_DISPLAY     = 0b00000001;
        const RETURN_HOME       = 0b00000010;
        const ENTRY_MODE        = 0b00000100;
        const DISPLAY_CONTROL   = 0b00001000;
        const SHIFT             = 0b00010000;
        const FUNCTION_SET      = 0b00100000;
        const SET_DDRAM         = 0b10000000;
    }
}

bitflags! {
    struct ShiftDirectionDirection: u8 {
        const CURSOR_MOVE_DECREMENT = 0b00000000;
        const CURSOR_MOVE_INCREMENT = 0b00000010;
    }
}

bitflags! {
    struct DisplayShift: u8 {
        const DISPLAY_SHIFT_DISABLE = 0b00000000;
        const DISPLAY_SHIFT_ENABLE  = 0b00000001;
    }
}

bitflags! {
    struct DisplayControl: u8 {
        // FIXME refactor same values
        const DISPLAY_OFF           = 0b00000000;
        const CURSOR_OFF            = 0b00000000;
        const CURSOR_BLINKING_OFF   = 0b00000000;
        const DISPLAY_ON            = 0b00000100;
        const CURSOR_ON             = 0b00000010;
        const CURSOR_BLINKING_ON    = 0b00000001;
    }
}

bitflags! {
    struct ShiftTarget: u8 {
        const CURSOR  = 0b00000000;
        const DISPLAY = 0b00001000;
    }
}

bitflags! {
    struct ShiftDirection: u8 {
        const RIGHT = 0b00000100;
        const LEFT  = 0b00000000;
    }
}

enum WriteMode {
    Command,
    Data,
}

pub struct DisplayPins {
    pub register_select: u64,
    pub enable: u64,
    pub data4: u64,
    pub data5: u64,
    pub data6: u64,
    pub data7: u64,
}

/// A HD44780 compliant display.
///
/// It provides a high-level and hardware agnostic interface to controll a HD44780 compliant
/// liquid crystal display (LCD).
pub struct Display<T: DisplayHardwareLayer> {
    register_select: T,
    enable: T,
    data4: T,
    data5: T,
    data6: T,
    data7: T,
    cursor_address: u8,
}
/// The `DisplayHardwareLayer` trait is intended to be implemented by the library user as a thin
/// wrapper around the hardware specific system calls.
pub trait DisplayHardwareLayer {
    /// Initializes an I/O pin.
    fn init(&self) {}
    /// Cleanup an I/O pin.
    fn cleanup(&self) {}
    /// Sets a value on an I/O pin.
    // TODO need a way to let the user set up how levels are interpreted by the hardware
    fn set_value(&self, u8) -> Result<(), ()>;
}

/// Enumeration of possible methods to move.
pub enum MoveDirection {
    // TODO rename to right, left
    /// Moves right.
    Increment,
    /// Moves left.
    Decrement,
}

/// Enumeration of possible methods to shift a cursor or display.
pub enum ShiftTo {
    /// Shifts to the right by the given offset.
    Right(u8),
    /// Shifts to the left by the given offset.
    Left(u8),
}

impl ShiftTo {
    fn into_offset_and_raw_direction(&self) -> (u8, ShiftDirection) {
        match *self {
            ShiftTo::Right(offset) => (offset, RIGHT),
            ShiftTo::Left(offset) => (offset, LEFT),
        }
    }
}

enum InnerSeekFrom {
    Home(u8),
    Current(u8),
    Line { address: u8, bytes: u8 },
}

/// Enumeration like struct of possible methods to seek within a `Display` object.
pub struct SeekFrom(InnerSeekFrom);

impl SeekFrom {
    /// Sets the cursor position to `Home` plus the provided number of bytes.
    pub fn home(bytes: u8) -> SeekFrom {
        SeekFrom(InnerSeekFrom::Home(bytes))
    }

    /// Sets the cursor to the current position plus the specified number of bytes.
    pub fn current(bytes: u8) -> SeekFrom {
        SeekFrom(InnerSeekFrom::Current(bytes))
    }

    /// Sets the cursor position to the provides line plus the specified number of bytes.
    pub fn line<T>(line: T, bytes: u8) -> SeekFrom
    where
        T: Addressable,
    {
        SeekFrom(InnerSeekFrom::Line {
            address: line.address(),
            bytes,
        })
    }
}

/// The `Addressable` trait provides a hardware address for a type.
pub trait Addressable {
    /// Returns the address of the type.
    fn address(&self) -> u8;
}

/// Enumeration of default lines.
pub enum DefaultLines {
    One,
    Two,
}

impl Addressable for DefaultLines {
    /// Returns the hardware address of the line.
    fn address(&self) -> u8 {
        match *self {
            DefaultLines::One => FIRST_LINE_ADDRESS,
            DefaultLines::Two => SECOND_LINE_ADDRESS,
        }
    }
}

/// A struct for creating display entry mode settings.
pub struct EntryModeBuilder {
    move_direction: MoveDirection,
    display_shift: bool,
}

impl EntryModeBuilder {
    /// Make a new `EntryModeBuilder` with the default settings described below.
    ///
    /// The default settings are:
    ///
    ///  - **move direction:**
    ///     - `Increment`
    ///  - **display_shift:**
    ///     - `Off`
    fn new() -> EntryModeBuilder {
        EntryModeBuilder {
            move_direction: MoveDirection::Increment,
            display_shift: false,
        }
    }

    /// Sets the direction the read/write cursor is moved when a character code is written to or
    /// read from the display.
    pub fn set_move_direction(&mut self, direction: MoveDirection) -> &mut EntryModeBuilder {
        self.move_direction = direction;
        self
    }

    /// Sets the display shift, which will be on character write, either `On` or `Off`.
    ///
    /// If display shift is enabled, it will seem as if the cursor does not move but the display
    /// does.
    ///
    /// **Note:** The display does not shift when reading.
    pub fn set_display_shift(&mut self, shift: bool) -> &mut EntryModeBuilder {
        self.display_shift = shift;
        self
    }

    fn build_command(&self) -> u8 {
        let mut cmd = ENTRY_MODE.bits();

        cmd |= match self.move_direction {
            MoveDirection::Increment => CURSOR_MOVE_INCREMENT.bits(),
            MoveDirection::Decrement => CURSOR_MOVE_DECREMENT.bits(),
        };

        cmd |= match self.display_shift {
            true => DISPLAY_SHIFT_ENABLE.bits(),
            false => DISPLAY_SHIFT_DISABLE.bits(),
        };

        cmd
    }
}

/// A struct for creating display control settings.
pub struct DisplayControlBuilder {
    // FIXME use enum instead of bool
    display: bool,
    cursor: bool,
    blink: bool,
}

impl DisplayControlBuilder {
    /// Makes a new `DisplayControlBuilder` using the default settings described below.
    ///
    /// The default settings are:
    ///
    ///  - **display:**
    ///     - `On`
    ///  - **cursor:**
    ///     - `Off`
    ///  - **blinking of cursor:**
    ///     - `Off`
    pub fn new() -> DisplayControlBuilder {
        DisplayControlBuilder {
            display: true,
            cursor: false,
            blink: false,
        }
    }

    /// Sets the entire display `On` or `Off`.
    ///
    /// Default is `On`.
    pub fn set_display(&mut self, status: bool) -> &mut DisplayControlBuilder {
        self.display = status;
        self
    }

    /// Sets the cursor `On` or `Off`.
    ///
    /// Default is `Off`.
    ///
    /// **Note:** This will not change cursor move direction or any other settings.
    pub fn set_cursor(&mut self, cursor: bool) -> &mut DisplayControlBuilder {
        self.cursor = cursor;
        self
    }

    /// Sets the blinking of the cursor `On` of `Off`.
    ///
    /// Default is `Off`.
    pub fn set_cursor_blinking(&mut self, blink: bool) -> &mut DisplayControlBuilder {
        self.blink = blink;
        self
    }

    fn build_command(&self) -> u8 {
        let mut cmd = DISPLAY_CONTROL.bits();

        cmd |= match self.display {
            true => DISPLAY_ON.bits(),
            false => DISPLAY_OFF.bits(),
        };

        cmd |= match self.cursor {
            true => CURSOR_ON.bits(),
            false => CURSOR_OFF.bits(),
        };

        cmd |= match self.cursor {
            true => CURSOR_BLINKING_ON.bits(),
            false => CURSOR_BLINKING_OFF.bits(),
        };

        cmd
    }
}

impl<T: From<u64> + DisplayHardwareLayer> Display<T> {
    /// Makes a new `Display` from a numeric pins configuration, given via `DisplayPins`.
    pub fn from_pins(pins: DisplayPins) -> Display<T> {
        let lcd = Display {
            register_select: T::from(pins.register_select),
            enable: T::from(pins.enable),
            data4: T::from(pins.data4),
            data5: T::from(pins.data5),
            data6: T::from(pins.data6),
            data7: T::from(pins.data7),
            cursor_address: 0,
        };

        lcd.register_select.init();
        lcd.enable.init();
        lcd.data4.init();
        lcd.data5.init();
        lcd.data6.init();
        lcd.data7.init();

        // Initializing by Instruction
        lcd.send_byte(0x33, WriteMode::Command);
        lcd.send_byte(0x32, WriteMode::Command);
        // FuctionSet: Data length 4bit + 2 lines
        lcd.send_byte(0x28, WriteMode::Command);
        // DisplayControl: Display on, Cursor off + cursor blinking off
        lcd.send_byte(0x0C, WriteMode::Command);
        // EntryModeSet: Cursor move direction inc + no display shift
        lcd.send_byte(0x06, WriteMode::Command);
        lcd.clear(); // ClearDisplay

        lcd
    }

    /// Sets the entry mode of the display using the builder given in the closure.
    pub fn set_entry_mode<F>(&self, f: F)
    where
        F: Fn(&mut EntryModeBuilder),
    {
        let mut builder = EntryModeBuilder::new();
        f(&mut builder);
        self.send_byte(builder.build_command(), WriteMode::Command);
    }

    /// Sets the display control settings using the builder given in the closure.
    pub fn set_display_control<F>(&self, f: F)
    where
        F: Fn(&mut DisplayControlBuilder),
    {
        let mut builder = DisplayControlBuilder::new();
        f(&mut builder);
        self.send_byte(builder.build_command(), WriteMode::Command);
    }

    /// Shifts the cursor to the left or the right by the given offset.
    ///
    /// **Note:** Consider to use [seek()](struct.Display.html#method.seek) for longer distances.
    pub fn shift_cursor(&mut self, direction: ShiftTo) {
        let (offset, raw_direction) = direction.into_offset_and_raw_direction();

        match direction {
            ShiftTo::Right(offset) => self.cursor_address += offset,
            ShiftTo::Left(offset) => self.cursor_address -= offset,
        }

        self.raw_shift(CURSOR, offset, raw_direction);
    }

    /// Shifts the display to the right or the left by the given offset.
    ///
    /// Note that the first and second line will shift at the same time.
    ///
    /// When the displayed data is shifted repeatedly each line moves only horizontally.
    /// The second line display does not shift into the first line position.
    pub fn shift(&self, direction: ShiftTo) {
        let (offset, raw_direction) = direction.into_offset_and_raw_direction();

        self.raw_shift(DISPLAY, offset, raw_direction);
    }

    fn raw_shift(&self, shift_type: ShiftTarget, offset: u8, raw_direction: ShiftDirection) {
        let mut cmd = SHIFT.bits();

        cmd |= shift_type.bits();
        cmd |= raw_direction.bits();

        for _ in 0..offset {
            self.send_byte(cmd, WriteMode::Command);
        }
    }

    /// Clears the entire display, sets the cursor to the home position and undo all display
    /// shifts.
    ///
    /// It also sets the cursor's move direction to `Increment`.
    pub fn clear(&self) {
        self.send_byte(CLEAR_DISPLAY.bits(), WriteMode::Command);
    }

    /// Seeks to an offset in display data RAM.
    pub fn seek(&mut self, pos: SeekFrom) {
        let mut cmd = SET_DDRAM.bits();

        let (start, bytes) = match pos.0 {
            InnerSeekFrom::Home(bytes) => (FIRST_LINE_ADDRESS, bytes),
            InnerSeekFrom::Current(bytes) => (self.cursor_address, bytes),
            InnerSeekFrom::Line { address, bytes } => (address, bytes),
        };

        self.cursor_address = start + bytes;

        cmd |= self.cursor_address;

        self.send_byte(cmd, WriteMode::Command);
    }

    fn send_byte(&self, value: u8, mode: WriteMode) {
        let wait_time = Duration::new(0, E_DELAY);

        match mode {
            WriteMode::Data => self.register_select.set_value(1),
            WriteMode::Command => self.register_select.set_value(0),
        }.unwrap();

        self.data4.set_value(0).unwrap();
        self.data5.set_value(0).unwrap();
        self.data6.set_value(0).unwrap();
        self.data7.set_value(0).unwrap();

        if value & 0x10 == 0x10 {
            self.data4.set_value(1).unwrap();
        }
        if value & 0x20 == 0x20 {
            self.data5.set_value(1).unwrap();
        }
        if value & 0x40 == 0x40 {
            self.data6.set_value(1).unwrap();
        }
        if value & 0x80 == 0x80 {
            self.data7.set_value(1).unwrap();
        }

        sleep(wait_time);
        self.enable.set_value(1).unwrap();
        sleep(wait_time);
        self.enable.set_value(0).unwrap();
        sleep(wait_time);

        self.data4.set_value(0).unwrap();
        self.data5.set_value(0).unwrap();
        self.data6.set_value(0).unwrap();
        self.data7.set_value(0).unwrap();

        if value & 0x01 == 0x01 {
            self.data4.set_value(1).unwrap();
        }
        if value & 0x02 == 0x02 {
            self.data5.set_value(1).unwrap();
        }
        if value & 0x04 == 0x04 {
            self.data6.set_value(1).unwrap();
        }
        if value & 0x08 == 0x08 {
            self.data7.set_value(1).unwrap();
        }

        sleep(wait_time);
        self.enable.set_value(1).unwrap();
        sleep(wait_time);
        self.enable.set_value(0).unwrap();
        sleep(wait_time);
    }

    /// Writes the given message on the display, starting from the current cursor position.
    pub fn write_message(&mut self, msg: &str) {
        for c in msg.as_bytes().iter().take(LCD_WIDTH) {
            self.cursor_address += 1;
            self.send_byte(*c, WriteMode::Data);
        }
    }
}

impl<T: DisplayHardwareLayer> Drop for Display<T> {
    fn drop(&mut self) {
        self.register_select.cleanup();
        self.enable.cleanup();
        self.data4.cleanup();
        self.data5.cleanup();
        self.data6.cleanup();
        self.data7.cleanup();
    }
}
