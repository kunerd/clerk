use core::marker::PhantomData;

use super::address::{Address, Overflow};
use super::{DisplayControlBuilder, EntryModeBuilder, FunctionSetBuilder, Home};
use hal::{Init, ReadMode, Receive, Send, SendInit, WriteMode};

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

pub enum DdRam {}
impl Overflow for DdRam {
    const UPPER_BOUND: u8 = 128;
}

pub enum CgRam {}
impl Overflow for CgRam {
    const UPPER_BOUND: u8 = 64;
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

pub type DdRamDisplay<P, U> = Display<P, U, DdRam>;

/// A HD44780 compliant display.
///
/// It provides a high-level and hardware agnostic interface to controll a HD44780 compliant
/// liquid crystal display (LCD).
pub struct Display<P, U, RT>
where
    U: Into<Address<RT>> + Home,
{
    connection: P,
    cursor_address: Address<RT>,
    _ram_type: PhantomData<RT>,
    _line_marker: PhantomData<U>,
}

impl<P, U> Display<P, U, DdRam>
where
    U: Into<Address<DdRam>> + Home,
{
    /// Create a new `Display` using the given connection.
    pub fn new(connection: P) -> Display<P, U, DdRam> {
        Display {
            connection: connection,
            cursor_address: Address::from(0),
            _ram_type: PhantomData::<DdRam>,
            _line_marker: PhantomData,
        }
    }
}

impl<P, U, RT> Display<P, U, RT>
where
    P: Init + Send + SendInit + Receive,
    U: Into<Address<RT>> + Home,
    RT: Overflow,
{
    // const FIRST_4BIT_INIT_INSTRUCTION: WriteMode = WriteMode::Command(0x33);
    // const SECOND_4BIT_INIT_INSTRUCTION: WriteMode = WriteMode::Command(0x32);

    pub fn init(&self, builder: &FunctionSetBuilder) {
        self.connection.init();

        let cmd = builder.build_command();
        let cmd = WriteMode::Command(cmd);

        self.init_by_instruction(cmd);
    }

    fn init_by_instruction(&self, function_set: WriteMode) {
        // self.connection.send(Self::FIRST_4BIT_INIT_INSTRUCTION);
        // self.connection.send(Self::SECOND_4BIT_INIT_INSTRUCTION);
        self.connection.send_init();

        self.connection.send(function_set);

        self.clear();
    }

    /// Sets the entry mode of the display.
    pub fn set_entry_mode(&self, builder: &EntryModeBuilder) {
        let cmd = WriteMode::Command(builder.build_command());
        self.connection.send(cmd);
    }

    /// Sets the display control settings.
    pub fn set_display_control(&self, builder: &DisplayControlBuilder) {
        let cmd = WriteMode::Command(builder.build_command());
        self.connection.send(cmd);
    }

    /// Shifts the cursor to the left or the right by the given offset.
    ///
    /// **Note:** Consider to use [seek()](struct.Display.html#method.seek) for longer distances.
    #[cfg_attr(feature = "cargo-clippy", allow(expl_impl_clone_on_copy))]
    pub fn shift_cursor(&mut self, direction: ShiftTo) {
        let (offset, raw_direction) = direction.as_offset_and_raw_direction();

        if offset == 0 {
            return;
        }

        match direction {
            ShiftTo::Right(offset) => self.cursor_address += offset.into(),
            ShiftTo::Left(offset) => self.cursor_address -= offset.into(),
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
            self.connection.send(WriteMode::Command(cmd));
        }
    }

    /// Clears the entire display, sets the cursor to the home position and undo all display
    /// shifts.
    ///
    /// It also sets the cursor's move direction to `Increment`.
    pub fn clear(&self) {
        let cmd = Instructions::CLEAR_DISPLAY.bits();
        self.connection.send(WriteMode::Command(cmd));

        // let (busy_flag, _) = self.read_busy_flag();
        // let mut busy_flag = busy_flag;

        // while busy_flag == true {
        //     let (bf, _) = self.read_busy_flag();
        //     busy_flag = bf;
        // }
    }

    /// Writes the given byte to data or character generator RAM, depending on the previous
    /// seek operation.
    pub fn write(&mut self, c: u8) {
        self.cursor_address += Address::from(1);
        self.connection.send(WriteMode::Data(c));
    }

    /// Reads a single byte from data RAM.
    pub fn read_byte(&mut self) -> u8 {
        self.cursor_address += Address::from(1);
        self.connection.receive(ReadMode::Data)
    }

    /// Reads busy flag and the cursor's current address.
    pub fn read_busy_flag(&self) -> (bool, u8) {
        let byte = self.connection.receive(ReadMode::BusyFlag);

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

    pub fn get_connection(self) -> P {
        self.connection
    }
}

/// Enumeration of possible methods to seek within the display data RAM (DDRAM).
pub enum SeekFrom<T>
where
    T: Into<Address<DdRam>>,
{
    /// Sets the cursor position to `Home` plus the provided number of bytes.
    Home(u8),
    /// Sets the cursor to the current position plus the specified number of bytes.
    Current(u8),
    /// Sets the cursor position to the given line plus the specified number of bytes.
    Line { line: T, offset: u8 },
}

/// Enumeration of possible methods to set an address in display data RAM (DDRAM).
pub enum SetFrom<T>
where
    T: Into<Address<DdRam>>,
{
    /// Sets the cursor position to `Home` plus the provided number of bytes.
    Home(u8),
    /// Sets the cursor position to the given line plus the specified number of bytes.
    Line { line: T, offset: u8 },
}

impl<T> From<SetFrom<T>> for SeekFrom<T>
where
    T: Into<Address<DdRam>>,
{
    fn from(pos: SetFrom<T>) -> Self {
        match pos {
            SetFrom::Home(offset) => SeekFrom::Home(offset),
            SetFrom::Line { line, offset } => SeekFrom::Line { line, offset },
        }
    }
}

impl<P, U> Display<P, U, DdRam>
where
    P: Send,
    U: Into<Address<DdRam>> + Into<Address<CgRam>> + Home,
{
    const SEEK_DDRAM_CMD: u8 = 0b1000_0000;

    /// Seeks to an offset in display data RAM.
    pub fn seek(&mut self, pos: SeekFrom<U>) {
        let mut cmd = Self::SEEK_DDRAM_CMD;

        let (start, addr) = match pos {
            SeekFrom::Home(offset) => (U::FIRST_LINE_ADDRESS.into(), offset.into()),
            SeekFrom::Current(offset) => (self.cursor_address, offset.into()),
            SeekFrom::Line { line, offset } => (line.into(), offset.into()),
        };

        self.cursor_address = start + addr;

        cmd |= u8::from(self.cursor_address);

        self.connection.send(WriteMode::Command(cmd));
    }

    /// Switches to the character generator RAM (CGRAM) and set the cursor's
    /// address to the given value. After that all following instructions will
    /// operate on this RAM type until it is switched back to display data RAM.
    pub fn set_cgram_address(self, address: u8) -> Display<P, U, CgRam> {
        let mut cgram_display = Display {
            connection: self.connection,
            cursor_address: Address::<CgRam>::from(0),
            _ram_type: PhantomData::<CgRam>,
            _line_marker: PhantomData,
        };

        cgram_display.seek(SeekCgRamFrom::Home(address));

        cgram_display
    }
}

/// Enumeration of possible methods to seek within the character generator RAM (CGRAM).
pub enum SeekCgRamFrom {
    /// Sets the cursor position to `Home` plus the provided number of bytes.
    Home(u8),
    /// Sets the cursor to the current position plus the specified number of bytes.
    Current(u8),
}

impl<P, U> Display<P, U, CgRam>
where
    P: Send,
    U: Into<Address<CgRam>> + Into<Address<DdRam>> + Home,
{
    const SEEK_CGRAM_CMD: u8 = 0b0100_0000;

    /// Seeks to an offset in character generator RAM.
    pub fn seek(&mut self, pos: SeekCgRamFrom) {
        let mut cmd = Self::SEEK_CGRAM_CMD;

        let addr = match pos {
            SeekCgRamFrom::Home(offset) => offset.into(),
            SeekCgRamFrom::Current(offset) => self.cursor_address + offset.into(),
        };

        self.cursor_address = addr;

        cmd |= u8::from(self.cursor_address);

        self.connection.send(WriteMode::Command(cmd));
    }

    /// Switches to the display data RAM (DDRAM) and set the cursor's address to
    /// the given value. After that all following instructions will operate on
    /// this RAM type until it is switched back to character generator RAM.
    pub fn set_ddram_address(self, pos: SetFrom<U>) -> Display<P, U, DdRam> {
        let mut ddram_display = Display {
            connection: self.connection,
            cursor_address: Address::from(0),
            _ram_type: PhantomData::<DdRam>,
            _line_marker: PhantomData,
        };

        ddram_display.seek(pos.into());

        ddram_display
    }
}
