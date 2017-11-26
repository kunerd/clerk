use core::marker::PhantomData;

use super::address::Address;
use super::{DisplayControlBuilder, EntryModeBuilder, FunctionSetBuilder, Home};

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

pub enum WriteMode {
    Command(u8),
    Data(u8),
}

pub enum ReadMode {
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
            ShiftTo::Right(offset) => (offset, ShiftDirection::RIGHT),
            ShiftTo::Left(offset) => (offset, ShiftDirection::LEFT),
        }
    }
}

/// Enumeration of possible methods to seek within a `Display` object.
pub enum SeekFrom<T: Into<Address>> {
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

pub trait Send {
    fn send(&self, mode: WriteMode);
}

pub trait SendRaw {
    fn send_byte(&self, byte: u8);
}

pub trait Recieve {
    fn recieve(&self, mode: ReadMode) -> u8;
}

pub trait RecieveRaw {
    fn receive_byte(&self) -> u8;
}

pub trait Init {
    fn init(&self);
}

pub struct Pins<RS, R, E, D> {
    pub register_select: RS,
    pub read: R,
    pub enable: E,
    pub data: D,
}

impl<RS, R, E, D> Init for Pins<RS, R, E, D>
where
    RS: DisplayHardwareLayer,
    R: DisplayHardwareLayer,
    E: DisplayHardwareLayer,
    D: Init,
{
    fn init(&self) {
        self.register_select.init();
        self.register_select.set_direction(Direction::Out);

        self.read.init();
        self.read.set_direction(Direction::Out);

        self.enable.init();
        self.enable.set_direction(Direction::Out);

        self.data.init();
    }
}

impl<RS, R, E, D> Send for Pins<RS, R, E, D>
where
    Self: SendRaw,
    RS: DisplayHardwareLayer,
    R: DisplayHardwareLayer,
{
    fn send(&self, mode: WriteMode) {
        self.read.set_level(Level::Low);

        let (level, value) = mode.into();
        self.register_select.set_level(level);

        self.send_byte(value);
    }
}

impl<RS, R, E, D> Recieve for Pins<RS, R, E, D>
where
    Self: RecieveRaw,
    RS: DisplayHardwareLayer,
    R: DisplayHardwareLayer,
{
    fn recieve(&self, mode: ReadMode) -> u8 {
        self.read.set_level(Level::High);

        match mode {
            ReadMode::Data => self.register_select.set_level(Level::High),
            ReadMode::BusyFlag => self.register_select.set_level(Level::Low),
        };

        self.receive_byte()
    }
}


fn get_bit(val: u8, bit: u8) -> Level {
    if val & bit == bit {
        Level::High
    } else {
        Level::Low
    }
}

// FIXME: WARNING - dummy implementation, not tested
impl<RS, R, E, P0, P1, P2, P3, P4, P5, P6, P7> SendRaw
    for Pins<RS, R, E, DataPins8Lines<P0, P1, P2, P3, P4, P5, P6, P7>>
where
    E: DisplayHardwareLayer,
    P0: DisplayHardwareLayer,
    P0: DisplayHardwareLayer,
    P1: DisplayHardwareLayer,
    P2: DisplayHardwareLayer,
    P3: DisplayHardwareLayer,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    fn send_byte(&self, byte: u8) {
        // D::delay_ns(D::ADDRESS_SETUP_TIME);
        self.enable.set_level(Level::High);

        self.data.data0.set_level(get_bit(byte, 0b0000_0001));
        self.data.data1.set_level(get_bit(byte, 0b0000_0010));
        self.data.data2.set_level(get_bit(byte, 0b0000_0100));
        self.data.data3.set_level(get_bit(byte, 0b0000_1000));
        self.data.data4.set_level(get_bit(byte, 0b0001_0000));
        self.data.data5.set_level(get_bit(byte, 0b0010_0000));
        self.data.data6.set_level(get_bit(byte, 0b0100_0000));
        self.data.data7.set_level(get_bit(byte, 0b1000_0000));

        // D::delay_ns(D::ENABLE_PULSE_WIDTH);
        self.enable.set_level(Level::Low);
        // D::delay_ns(D::DATA_HOLD_TIME);
    }
}

pub struct DataPins8Lines<P0, P1, P2, P3, P4, P5, P6, P7>
where
    P0: DisplayHardwareLayer,
    P1: DisplayHardwareLayer,
    P2: DisplayHardwareLayer,
    P3: DisplayHardwareLayer,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    pub data0: P0,
    pub data1: P1,
    pub data2: P2,
    pub data3: P3,
    pub data4: P4,
    pub data5: P5,
    pub data6: P6,
    pub data7: P7,
}

impl<P0, P1, P2, P3, P4, P5, P6, P7> Init for DataPins8Lines<P0, P1, P2, P3, P4, P5, P6, P7>
where
    P0: DisplayHardwareLayer,
    P0: DisplayHardwareLayer,
    P1: DisplayHardwareLayer,
    P2: DisplayHardwareLayer,
    P3: DisplayHardwareLayer,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    fn init(&self) {
        // TODO maybe not needed because of pin state config
        self.data0.init();
        self.data1.init();
        self.data2.init();
        self.data3.init();
        self.data4.init();
        self.data5.init();
        self.data6.init();
        self.data7.init();
    }
}

pub struct DataPins4Lines<P4, P5, P6, P7>
where
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    pub data4: P4,
    pub data5: P5,
    pub data6: P6,
    pub data7: P7,
}

impl<P4, P5, P6, P7> Init for DataPins4Lines<P4, P5, P6, P7>
where
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    fn init(&self) {
        self.data4.init();
        self.data5.init();
        self.data6.init();
        self.data7.init();
    }
}

impl<RS, R, E, P4, P5, P6, P7> SendRaw for Pins<RS, R, E, DataPins4Lines<P4, P5, P6, P7>>
where
    E: DisplayHardwareLayer,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    fn send_byte(&self, byte: u8) {
        self.data.data4.set_direction(Direction::Out);
        self.data.data5.set_direction(Direction::Out);
        self.data.data6.set_direction(Direction::Out);
        self.data.data7.set_direction(Direction::Out);

        write_4bit(self, Nibble::Upper(byte));
        write_4bit(self, Nibble::Lower(byte));
    }
}

fn write_4bit<RS, R, E, P4, P5, P6, P7>(
    pins: &Pins<RS, R, E, DataPins4Lines<P4, P5, P6, P7>>,
    nibble: Nibble,
) where
    E: DisplayHardwareLayer,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    let value: u8 = nibble.into();

    // D::delay_ns(D::ADDRESS_SETUP_TIME);
    pins.enable.set_level(Level::High);

    if value & 0x01 == 0x01 {
        pins.data.data4.set_level(Level::High);
    } else {
        pins.data.data4.set_level(Level::Low);
    }

    if value & 0x02 == 0x02 {
        pins.data.data5.set_level(Level::High);
    } else {
        pins.data.data5.set_level(Level::Low);
    }

    if value & 0x04 == 0x04 {
        pins.data.data6.set_level(Level::High);
    } else {
        pins.data.data6.set_level(Level::Low);
    }

    if value & 0x08 == 0x08 {
        pins.data.data7.set_level(Level::High);
    } else {
        pins.data.data7.set_level(Level::Low);
    }

    // D::delay_ns(D::ENABLE_PULSE_WIDTH);
    pins.enable.set_level(Level::Low);
    // D::delay_ns(D::DATA_HOLD_TIME);
}

impl<RS, R, E, P4, P5, P6, P7> RecieveRaw for Pins<RS, R, E, DataPins4Lines<P4, P5, P6, P7>>
where
    E: DisplayHardwareLayer,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    fn receive_byte(&self) -> u8 {
        self.data.data4.set_direction(Direction::In);
        self.data.data5.set_direction(Direction::In);
        self.data.data6.set_direction(Direction::In);
        self.data.data7.set_direction(Direction::In);

        let upper = read_single_nibble(self);
        let lower = read_single_nibble(self);

        let mut result = upper << 4;
        result |= lower & 0x0f;

        result
    }
}

fn read_single_nibble<RS, R, E, P4, P5, P6, P7>(
    pins: &Pins<RS, R, E, DataPins4Lines<P4, P5, P6, P7>>,
) -> u8
where
    E: DisplayHardwareLayer,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    let mut result = 0u8;

    // D::delay_ns(D::ADDRESS_SETUP_TIME);
    pins.enable.set_level(Level::High);

    result |= pins.data.data7.get_value() << 3;
    result |= pins.data.data6.get_value() << 2;
    result |= pins.data.data5.get_value() << 1;
    result |= pins.data.data4.get_value();

    // D::delay_ns(D::ENABLE_PULSE_WIDTH);
    pins.enable.set_level(Level::Low);
    // D::delay_ns(D::DATA_HOLD_TIME);

    result
}

/// A HD44780 compliant display.
///
/// It provides a high-level and hardware agnostic interface to controll a HD44780 compliant
/// liquid crystal display (LCD).
pub struct Display<P, U>
where
    U: Into<Address> + Home,
{
    pins: P,
    cursor_address: Address,
    _line_marker: PhantomData<U>,
    // _delay_marker: PhantomData<D>,
}

impl<P, U> Display<P, U>
where
    P: Init + Send + Recieve,
    U: Into<Address> + Home,
{
    const FIRST_4BIT_INIT_INSTRUCTION: WriteMode = WriteMode::Command(0x33);
    const SECOND_4BIT_INIT_INSTRUCTION: WriteMode = WriteMode::Command(0x32);

    /// Makes a new `Display` from a numeric pins configuration, given via `DisplayPins`.
    pub fn from_pins(pins: P) -> Self {
        Display {
            pins: pins,
            cursor_address: Address::from(0),
            _line_marker: PhantomData,
            // _delay_marker: PhantomData,
        }
    }

    fn init_by_instruction(&self, function_set: WriteMode) {
        self.pins.send(Self::FIRST_4BIT_INIT_INSTRUCTION);
        self.pins.send(Self::SECOND_4BIT_INIT_INSTRUCTION);

        self.pins.send(function_set);

        self.clear();
    }

    pub fn init(&self, builder: &FunctionSetBuilder) {
        self.pins.init();

        let cmd = builder.build_command();
        let cmd = WriteMode::Command(cmd);

        self.init_by_instruction(cmd);
    }

    /// Sets the entry mode of the display.
    pub fn set_entry_mode(&self, builder: &EntryModeBuilder) {
        let cmd = WriteMode::Command(builder.build_command());
        self.pins.send(cmd);
    }

    /// Sets the display control settings.
    pub fn set_display_control(&self, builder: &DisplayControlBuilder) {
        let cmd = WriteMode::Command(builder.build_command());
        self.pins.send(cmd);
    }

    /// Shifts the cursor to the left or the right by the given offset.
    ///
    /// **Note:** Consider to use [seek()](struct.Display.html#method.seek) for longer distances.
    pub fn shift_cursor(&mut self, direction: ShiftTo) {
        let (offset, raw_direction) = direction.as_offset_and_raw_direction();

        if offset == 0 {
            return;
        }

        match direction {
            ShiftTo::Right(offset) => self.cursor_address += offset.into(),
            ShiftTo::Left(offset) => self.cursor_address -= offset.into()
        }

        self.raw_shift(ShiftTarget::CURSOR, offset, raw_direction);
    }

    /// Shifts the display to the right or the left by the given offset.
    ///
    /// Note that the first and second line will shift at the same time.
    ///
    /// When the displayed data is shifted repeatedly each line moves only horizontally.
    /// The second line display does not shift into the first line position.
    pub fn shift(&self, direction: ShiftTo) {
        let (offset, raw_direction) = direction.as_offset_and_raw_direction();

        self.raw_shift(ShiftTarget::DISPLAY, offset, raw_direction);
    }

    fn raw_shift(&self, shift_type: ShiftTarget, offset: u8, raw_direction: ShiftDirection) {
        let mut cmd = Instructions::SHIFT.bits();

        cmd |= shift_type.bits();
        cmd |= raw_direction.bits();

        for _ in 0..offset {
            self.pins.send(WriteMode::Command(cmd));
        }
    }

    /// Clears the entire display, sets the cursor to the home position and undo all display
    /// shifts.
    ///
    /// It also sets the cursor's move direction to `Increment`.
    pub fn clear(&self) {
        let cmd = Instructions::CLEAR_DISPLAY.bits();
        self.pins.send(WriteMode::Command(cmd));
    }

    fn generic_seek(&mut self, ram_type: RamType, pos: SeekFrom<U>) {
        let mut cmd = ram_type.into();

        let (start, addr) = match pos {
            SeekFrom::Home(bytes) => (U::FIRST_LINE_ADDRESS.into(), bytes.into()),
            SeekFrom::Current(bytes) => (self.cursor_address, bytes.into()),
            SeekFrom::Line { line, bytes } => (line.into(), bytes.into()),
        };

        self.cursor_address = start + addr;

        cmd |= u8::from(self.cursor_address);

        self.pins.send(WriteMode::Command(cmd));
    }

    /// Seeks to an offset in display data RAM.
    pub fn seek(&mut self, pos: SeekFrom<U>) {
        self.generic_seek(RamType::DisplayData, pos);
    }

    /// Seeks to an offset in display character generator RAM.
    pub fn seek_cgram(&mut self, pos: SeekFrom<U>) {
        self.generic_seek(RamType::CharacterGenerator, pos);
    }

    /// Writes the given byte to data or character generator RAM, depending on the previous
    /// seek operation.
    pub fn write(&mut self, c: u8) {
        self.cursor_address += Address::from(1);
        self.pins.send(WriteMode::Data(c));
    }

    /// Reads a single byte from data RAM.
    pub fn read_byte(&mut self) -> u8 {
        self.cursor_address += Address::from(1);
        self.pins.recieve(ReadMode::Data)
    }

    /// Reads busy flag and the cursor's current address.
    pub fn read_busy_flag(&self) -> (bool, u8) {
        let byte = self.pins.recieve(ReadMode::BusyFlag);

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

    pub fn get_pins(self) -> P {
        self.pins
    }
}
