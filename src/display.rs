use core::marker::PhantomData;

// TODO replace by configurable value
use super::FIRST_LINE_ADDRESS;
use super::{DisplayControlBuilder, EntryModeBuilder, FunctionSetBuilder};

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
    /// Sets the cursor position to the given line plus the specified number of bytes.
    Line { line: T, bytes: u8 },
}

/// Enumeration of possible data directions of a pin.
#[derive(Debug, PartialEq)]
pub enum Direction {
    In,
    Out,
}

enum Nibble {
    Upper(u8),
    Lower(u8),
}

impl From<Nibble> for u8 {
    fn from(n: Nibble) -> Self {
        match n {
            Nibble::Upper(base) => base >> 4,
            Nibble::Lower(base) => base & 0x0f,
        }
    }
}

/// Enumeration of possible levels of a pin.
#[derive(Debug, PartialEq)]
pub enum Level {
    Low,
    High,
}

impl From<WriteMode> for (Level, u8) {
    fn from(mode: WriteMode) -> Self {
        match mode {
            WriteMode::Command(value) => (Level::Low, value),
            WriteMode::Data(value) => (Level::High, value),
        }
    }
}

/// The `Delay` trait is used to adapt the timing to the specific hardware and must be implemented
/// by the libary user.
pub trait Delay {
    /// The time (ns) between register select (RS) and read/write (R/W) to enable signal (E).
    const ADDRESS_SETUP_TIME: u16 = 60;
    /// The duration (ns) the enable signal is set to `High`.
    const ENABLE_PULSE_WIDTH: u16 = 450;
    /// The duration (ns) the data pins will be set after the enable signal was dropped.
    const DATA_HOLD_TIME: u16 = 20;

    /// The maximum execution time of instruction commands.
    const COMMAND_EXECUTION_TIME: u16 = 37;

    /// Wait for the given amount of nanoseconds.
    fn delay_ns(ns: u16);

    /// Wait for the given amount of microseconds.
    fn delay_us(us: u16) {
        for _ in 0..us {
            Self::delay_ns(1000);
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
    fn set_level(&self, Level);

    fn get_value(&self) -> u8;
}

pub struct DisplayPins<T>
where
    T: DisplayHardwareLayer,
{
    pub register_select: T,
    pub read: T,
    pub enable: T,
    pub data4: T,
    pub data5: T,
    pub data6: T,
    pub data7: T,
}

/// A HD44780 compliant display.
///
/// It provides a high-level and hardware agnostic interface to controll a HD44780 compliant
/// liquid crystal display (LCD).
pub struct Display<T, U, D>
where
    T: DisplayHardwareLayer,
    U: Into<u8>,
    D: Delay,
{
    pins: DisplayPins<T>,
    cursor_address: u8,
    _line_marker: PhantomData<U>,
    _delay_marker: PhantomData<D>,
}

impl<T, U, D> Display<T, U, D>
where
    T: DisplayHardwareLayer,
    U: Into<u8>,
    D: Delay,
{
    const FIRST_4BIT_INIT_INSTRUCTION: WriteMode = WriteMode::Command(0x33);
    const SECOND_4BIT_INIT_INSTRUCTION: WriteMode = WriteMode::Command(0x32);

    /// Makes a new `Display` from a numeric pins configuration, given via `DisplayPins`.
    pub fn from_pins(pins: DisplayPins<T>) -> Display<T, U, D> {
        Display {
            pins: pins,
            cursor_address: 0,
            _line_marker: PhantomData,
            _delay_marker: PhantomData,
        }
    }

    fn init_pins(&self) {
        self.pins.register_select.init();
        self.pins.register_select.set_direction(Direction::Out);

        self.pins.read.init();
        self.pins.read.set_direction(Direction::Out);

        self.pins.enable.init();
        self.pins.enable.set_direction(Direction::Out);

        // TODO maybe not needed because of pin state config
        self.pins.data4.init();
        self.pins.data5.init();
        self.pins.data6.init();
        self.pins.data7.init();
    }

    fn init_by_instruction(&self, function_set: WriteMode) {
        self.write_byte(Self::FIRST_4BIT_INIT_INSTRUCTION);
        self.write_byte(Self::SECOND_4BIT_INIT_INSTRUCTION);

        self.write_byte(function_set);

        self.clear();
    }

    pub fn init(&self, builder: &FunctionSetBuilder) {
        self.init_pins();

        let cmd = builder.build_command();
        let cmd = WriteMode::Command(cmd);

        self.init_by_instruction(cmd);
    }

    /// Sets the entry mode of the display.
    pub fn set_entry_mode(&self, builder: &EntryModeBuilder) {
        let cmd = WriteMode::Command(builder.build_command());
        self.write_byte(cmd);
    }

    /// Sets the display control settings.
    pub fn set_display_control(&self, builder: &DisplayControlBuilder) {
        let cmd = WriteMode::Command(builder.build_command());
        self.write_byte(cmd);
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

        D::delay_ns(D::ADDRESS_SETUP_TIME);
        self.pins.enable.set_level(Level::High);

        if value & 0x01 == 0x01 {
            self.pins.data4.set_level(Level::High);
        } else {
            self.pins.data4.set_level(Level::Low);
        }

        if value & 0x02 == 0x02 {
            self.pins.data5.set_level(Level::High);
        } else {
            self.pins.data5.set_level(Level::Low);
        }

        if value & 0x04 == 0x04 {
            self.pins.data6.set_level(Level::High);
        } else {
            self.pins.data6.set_level(Level::Low);
        }

        if value & 0x08 == 0x08 {
            self.pins.data7.set_level(Level::High);
        } else {
            self.pins.data7.set_level(Level::Low);
        }

        D::delay_ns(D::ENABLE_PULSE_WIDTH);
        self.pins.enable.set_level(Level::Low);
        D::delay_ns(D::DATA_HOLD_TIME);
    }

    fn send_byte(&self, byte: u8) {
        self.write_4bit(Nibble::Upper(byte));
        self.write_4bit(Nibble::Lower(byte));
    }

    fn write_byte(&self, mode: WriteMode) {
        self.pins.data4.set_direction(Direction::Out);
        self.pins.data5.set_direction(Direction::Out);
        self.pins.data6.set_direction(Direction::Out);
        self.pins.data7.set_direction(Direction::Out);

        self.pins.read.set_level(Level::Low);

        let (level, value) = mode.into();
        self.pins.register_select.set_level(level);

        self.send_byte(value);
    }

    /// Writes the given byte to data or character generator RAM, depending on the previous
    /// seek operation.
    pub fn write(&mut self, c: u8) {
        self.cursor_address += 1;
        self.write_byte(WriteMode::Data(c));
    }

    fn read_single_nibble(&self) -> u8 {
        let mut result = 0u8;

        D::delay_ns(D::ADDRESS_SETUP_TIME);
        self.pins.enable.set_level(Level::High);

        result |= self.pins.data7.get_value() << 3;
        result |= self.pins.data6.get_value() << 2;
        result |= self.pins.data5.get_value() << 1;
        result |= self.pins.data4.get_value();

        D::delay_ns(D::ENABLE_PULSE_WIDTH);
        self.pins.enable.set_level(Level::Low);
        D::delay_ns(D::DATA_HOLD_TIME);

        result
    }

    fn receive_byte(&self) -> u8 {
        let upper = self.read_single_nibble();
        let lower = self.read_single_nibble();

        let mut result = upper << 4;
        result |= lower & 0x0f;

        result
    }

    fn read_raw_byte(&self, mode: ReadMode) -> u8 {
        self.pins.data4.set_direction(Direction::In);
        self.pins.data5.set_direction(Direction::In);
        self.pins.data6.set_direction(Direction::In);
        self.pins.data7.set_direction(Direction::In);

        self.pins.read.set_level(Level::High);

        match mode {
            ReadMode::Data => self.pins.register_select.set_level(Level::High),
            ReadMode::BusyFlag => self.pins.register_select.set_level(Level::Low),
        };

        self.receive_byte()
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
            self.write(*c);
        }
    }

    pub fn get_pins(self) -> DisplayPins<T> {
        self.pins
    }
}
