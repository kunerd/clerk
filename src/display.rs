use std::marker::PhantomData;
use std::thread::sleep;
use std::time::Duration;

// TODO replace by configurable value
use super::FIRST_LINE_ADDRESS;
use super::{DisplayControlBuilder, EntryModeBuilder};

// TODO make configurable
// TODO add optional implementation using the busy flag
static E_DELAY: u32 = 5;
const LCD_WIDTH: usize = 16;

bitflags! {
    struct Instructions: u8 {
        const CLEAR_DISPLAY     = 0b00000001;
        const RETURN_HOME       = 0b00000010;
        const SHIFT             = 0b00010000;
        const SET_DDRAM         = 0b10000000;
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

/// Enumeration of possible methods to shift a cursor or display.
pub enum ShiftTo {
    /// Shifts to the right by the given offset.
    Right(u8),
    /// Shifts to the left by the given offset.
    Left(u8),
}

impl ShiftTo {
    fn as_offset_and_raw_direction(&self) -> (u8, ShiftDirection) {
        match *self {
            ShiftTo::Right(offset) => (offset, RIGHT),
            ShiftTo::Left(offset) => (offset, LEFT),
        }
    }
}

/// Enumeration of possible methods to seek within a `Display` object.
pub enum SeekFrom<T: Into<u8>> {
    /// Sets the cursor position to `Home` plus the provided number of bytes.
    Home(u8),
    /// Sets the cursor to the current position plus the specified number of bytes.
    Current(u8),
    /// Sets the cursor position to the provides line plus the specified number of bytes.
    Line { line: T, bytes: u8 },
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
pub struct Display<T, U>
where
    T: From<u64> + DisplayHardwareLayer,
    U: Into<u8>,
{
    register_select: T,
    enable: T,
    data4: T,
    data5: T,
    data6: T,
    data7: T,
    cursor_address: u8,
    _marker: PhantomData<U>,
}

impl<T, U> Display<T, U>
where
    T: From<u64> + DisplayHardwareLayer,
    U: Into<u8>,
{
    /// Makes a new `Display` from a numeric pins configuration, given via `DisplayPins`.
    pub fn from_pins(pins: DisplayPins) -> Display<T, U> {
        let lcd = Display {
            register_select: T::from(pins.register_select),
            enable: T::from(pins.enable),
            data4: T::from(pins.data4),
            data5: T::from(pins.data5),
            data6: T::from(pins.data6),
            data7: T::from(pins.data7),
            cursor_address: 0,
            _marker: PhantomData,
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
        let mut builder = EntryModeBuilder::default();
        f(&mut builder);
        self.send_byte(builder.build_command(), WriteMode::Command);
    }

    /// Sets the display control settings using the builder given in the closure.
    pub fn set_display_control<F>(&self, f: F)
    where
        F: Fn(&mut DisplayControlBuilder),
    {
        let mut builder = DisplayControlBuilder::default();
        f(&mut builder);
        self.send_byte(builder.build_command(), WriteMode::Command);
    }

    /// Shifts the cursor to the left or the right by the given offset.
    ///
    /// **Note:** Consider to use [seek()](struct.Display.html#method.seek) for longer distances.
    pub fn shift_cursor(&mut self, direction: ShiftTo) {
        let (offset, raw_direction) = direction.as_offset_and_raw_direction();

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
        let (offset, raw_direction) = direction.as_offset_and_raw_direction();

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
    pub fn seek(&mut self, pos: SeekFrom<U>) {
        let mut cmd = SET_DDRAM.bits();

        let (start, bytes) = match pos {
            SeekFrom::Home(bytes) => (FIRST_LINE_ADDRESS, bytes),
            SeekFrom::Current(bytes) => (self.cursor_address, bytes),
            SeekFrom::Line { line, bytes } => (line.into(), bytes),
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

impl<T, U> Drop for Display<T, U>
where
    T: From<u64> + DisplayHardwareLayer,
    U: Into<u8>,
{
    fn drop(&mut self) {
        self.register_select.cleanup();
        self.enable.cleanup();
        self.data4.cleanup();
        self.data5.cleanup();
        self.data6.cleanup();
        self.data7.cleanup();
    }
}