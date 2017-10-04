use core::marker::PhantomData;

// TODO replace by configurable value
use super::FIRST_LINE_ADDRESS;
use super::{DisplayControlBuilder, EntryModeBuilder};

const LCD_WIDTH: usize = 16;

bitflags! {
    struct Instructions: u8 {
        const CLEAR_DISPLAY     = 0b0000_0001;
        const RETURN_HOME       = 0b0000_0010;
        const SHIFT             = 0b0001_0000;
    }
}

bitflags! {
    struct ShiftTarget: u8 {
        const CURSOR  = 0b0000_0000;
        const DISPLAY = 0b0000_1000;
    }
}

bitflags! {
    struct ShiftDirection: u8 {
        const RIGHT = 0b0000_0100;
        const LEFT  = 0b0000_0000;
    }
}

enum WriteMode {
    Command(u8),
    Data(u8),
}

enum ReadMode {
    Data,
    // TODO: use busy flag
    BusyFlag,
}

enum RamType {
    DisplayData,
    CharacterGenerator,
}

impl From<RamType> for u8 {
    fn from(ram_type: RamType) -> Self {
        match ram_type {
            RamType::DisplayData => 0b1000_0000,
            RamType::CharacterGenerator => 0b0100_0000,
        }
    }
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

/// Enumeration of possible data directions of a pin.
pub enum Direction {
    In,
    Out,
}

enum Nibble {
    Upper(u8),
    Lower(u8)
}

impl From<Nibble> for u8 {
    fn from(n: Nibble) -> Self {
        match n {
            Nibble::Upper(base) => base >> 4,
            Nibble::Lower(base) => base & 0x0f
        }
    }
}

/// The `DisplayHardwareLayer` trait is intended to be implemented by the library user as a thin
/// wrapper around the hardware specific system calls.
pub trait DisplayHardwareLayer {
    /// Initializes an I/O pin.
    fn init(&self) {}
    /// Cleanup an I/O pin.
    fn cleanup(&self) {}

    fn set_direction(&self, Direction);
    /// Sets a value on an I/O pin.
    // TODO need a way to let the user set up how levels are interpreted by the hardware
    fn set_value(&self, u8) -> Result<(), ()>;

    fn get_value(&self) -> u8;
}

pub struct DisplayPins {
    pub register_select: u64,
    pub read: u64,
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
    read: T,
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
            read: T::from(pins.read),
            data4: T::from(pins.data4),
            data5: T::from(pins.data5),
            data6: T::from(pins.data6),
            data7: T::from(pins.data7),
            cursor_address: 0,
            _marker: PhantomData,
        };

        lcd.register_select.init();
        lcd.read.init();
        lcd.enable.init();
        lcd.data4.init();
        lcd.data5.init();
        lcd.data6.init();
        lcd.data7.init();

        lcd.read.set_value(0).unwrap();

        // FIXME remove magic numbers
        // Initializing by Instruction
        lcd.write_byte(WriteMode::Command(0x33));
        lcd.write_byte(WriteMode::Command(0x32));
        // FuctionSet: Data length 4bit + 2 lines
        lcd.write_byte(WriteMode::Command(0x28));
        // DisplayControl: Display on, Cursor off + cursor blinking off
        lcd.write_byte(WriteMode::Command(0x0C));
        // EntryModeSet: Cursor move direction inc + no display shift
        lcd.write_byte(WriteMode::Command(0x06));
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
        self.write_byte(WriteMode::Command(builder.build_command()));
    }

    /// Sets the display control settings using the builder given in the closure.
    pub fn set_display_control<F>(&self, f: F)
    where
        F: Fn(&mut DisplayControlBuilder),
    {
        let mut builder = DisplayControlBuilder::default();
        f(&mut builder);
        self.write_byte(WriteMode::Command(builder.build_command()));
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
            self.write_byte(WriteMode::Command(cmd));
        }
    }

    /// Clears the entire display, sets the cursor to the home position and undo all display
    /// shifts.
    ///
    /// It also sets the cursor's move direction to `Increment`.
    pub fn clear(&self) {
        let cmd = CLEAR_DISPLAY.bits();
        self.write_byte(WriteMode::Command(cmd));
    }

    fn generic_seek(&mut self, ram_type: RamType, pos: SeekFrom<U>) {
        let mut cmd = ram_type.into();

        let (start, bytes) = match pos {
            SeekFrom::Home(bytes) => (FIRST_LINE_ADDRESS, bytes),
            SeekFrom::Current(bytes) => (self.cursor_address, bytes),
            SeekFrom::Line { line, bytes } => (line.into(), bytes),
        };

        self.cursor_address = start + bytes;

        cmd |= self.cursor_address;

        self.write_byte(WriteMode::Command(cmd));
    }

    /// Seeks to an offset in display data RAM.
    pub fn seek(&mut self, pos: SeekFrom<U>) {
        self.generic_seek(RamType::DisplayData, pos);
    }

    /// Seeks to an offset in display character generator RAM.
    pub fn seek_cgram(&mut self, pos: SeekFrom<U>) {
        self.generic_seek(RamType::CharacterGenerator, pos);
    }

    fn write_4bit(&self, nibble: Nibble) {
        let value: u8 = nibble.into();

        self.data4.set_value(0).unwrap();
        self.data5.set_value(0).unwrap();
        self.data6.set_value(0).unwrap();
        self.data7.set_value(0).unwrap();

        // FIXME: add delay
        self.enable.set_value(1).unwrap();

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

        // FIXME: add delay
        self.enable.set_value(0).unwrap();
        // FIXME: add delay
    }

    fn send_byte(&self, byte: u8) {
        self.write_4bit(Nibble::Upper(byte));
        self.write_4bit(Nibble::Lower(byte));
    }

    fn write_data(&self, value: u8) {
        self.register_select.set_value(1).unwrap();

        self.send_byte(value);
    }

    fn write_command(&self, cmd: u8) {
        self.register_select.set_value(0).unwrap();

        self.send_byte(cmd);
    }

    fn write_byte(&self, mode: WriteMode) {
        self.data4.set_direction(Direction::Out);
        self.data5.set_direction(Direction::Out);
        self.data6.set_direction(Direction::Out);
        self.data7.set_direction(Direction::Out);

        self.read.set_value(0).unwrap();

        match mode {
            WriteMode::Data(value) => self.write_data(value),
            WriteMode::Command(cmd) => self.write_command(cmd),
        }
    }

    fn read_raw_byte(&self, mode: ReadMode) -> u8 {
        let mut result = 0u8;

        self.data4.set_direction(Direction::In);
        self.data5.set_direction(Direction::In);
        self.data6.set_direction(Direction::In);
        self.data7.set_direction(Direction::In);

        match mode {
            ReadMode::Data => self.register_select.set_value(1),
            ReadMode::BusyFlag => self.register_select.set_value(0),
        }.unwrap();

        self.read.set_value(1).unwrap();
        // FIXME: add delay, 45ms
        self.enable.set_value(1).unwrap();
        // FIXME: add delay, 165ms

        result |= self.data7.get_value() << 7;
        result |= self.data6.get_value() << 6;
        result |= self.data5.get_value() << 5;
        result |= self.data4.get_value() << 4;

        self.enable.set_value(0).unwrap();
        // FIXME: add delay, 45ms
        self.enable.set_value(1).unwrap();
        // FIXME: add delay, 165ms

        result |= self.data7.get_value() << 3;
        result |= self.data6.get_value() << 2;
        result |= self.data5.get_value() << 1;
        result |= self.data4.get_value();

        self.enable.set_value(0).unwrap();
        // FIXME: add delay, 45ms

        result
    }

    /// Reads a single byte from data RAM.
    pub fn read_byte(&mut self) -> u8 {
        self.cursor_address += 1;
        self.read_raw_byte(ReadMode::Data)
    }

    /// Reads busy flag and the cursor's current address.
    pub fn read_busy_flag(&self) -> (bool, u8) {
        let byte = self.read_raw_byte(ReadMode::BusyFlag);

        let busy_flag = (byte & 0b1000_0000) != 0;

        let address = byte & 0b0111_1111;

        (busy_flag, address)
    }

    /// Writes the given message to data or character generator RAM, depending on the previous
    /// seek operation.
    pub fn write_message(&mut self, msg: &str) {
        for c in msg.as_bytes().iter().take(LCD_WIDTH) {
            self.cursor_address += 1;

            self.write_byte(WriteMode::Data(*c));
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
