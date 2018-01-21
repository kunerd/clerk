use core::marker::PhantomData;

use super::address::Address;
use super::{DisplayControlBuilder, EntryModeBuilder, FunctionSetBuilder, Home};
use hal::{Init, ReadMode, Receive, Send, WriteMode};

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

pub struct CgRam;
pub struct DdRam;

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

/// A HD44780 compliant display.
///
/// It provides a high-level and hardware agnostic interface to controll a HD44780 compliant
/// liquid crystal display (LCD).
pub struct Display<P, U, RT>
where
    U: Into<Address> + Home,
{
    connection: P,
    cursor_address: Address,
    _ram_type: PhantomData<RT>,
    _line_marker: PhantomData<U>,
}

impl<P, U> Display<P, U, DdRam>
where
    U: Into<Address> + Home,
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
    P: Init + Send + Receive,
    U: Into<Address> + Home,
{
    const FIRST_4BIT_INIT_INSTRUCTION: WriteMode = WriteMode::Command(0x33);
    const SECOND_4BIT_INIT_INSTRUCTION: WriteMode = WriteMode::Command(0x32);

    pub fn init(&self, builder: &FunctionSetBuilder) {
        self.connection.init();

        let cmd = builder.build_command();
        let cmd = WriteMode::Command(cmd);

        self.init_by_instruction(cmd);
    }

    fn init_by_instruction(&self, function_set: WriteMode) {
        self.connection.send(Self::FIRST_4BIT_INIT_INSTRUCTION);
        self.connection.send(Self::SECOND_4BIT_INIT_INSTRUCTION);

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


impl<P, U> Display<P, U, DdRam>
where
    U: Into<Address> + Home,
    P: Send,
{
    /// Seeks to an offset in display data RAM.
    pub fn seek(&mut self, pos: SeekFrom<U>) {
        // FIXME remove magic number
        let mut cmd = 0b1000_0000;

        let (start, addr) = match pos {
            SeekFrom::Home(bytes) => (U::FIRST_LINE_ADDRESS.into(), bytes.into()),
            SeekFrom::Current(bytes) => (self.cursor_address, bytes.into()),
            SeekFrom::Line { line, bytes } => (line.into(), bytes.into()),
        };

        self.cursor_address = start + addr;

        cmd |= u8::from(self.cursor_address);

        self.connection.send(WriteMode::Command(cmd));
    }

    pub fn seek_cgram(self, address: u8) -> Display<P, U, CgRam> {
        let mut cgram_display = Display {
            connection: self.connection,
            cursor_address: Address::from(0),
            _ram_type: PhantomData::<CgRam>,
            _line_marker: PhantomData,
        };

        cgram_display.seek(SeekFrom::Home(address));

        cgram_display
    }
}

impl<P, U> Display<P, U, CgRam>
where
    U: Into<Address> + Home,
    P: Send,
{
    /// Seeks to an offset in display data RAM.
    // FIXME seek from character address, character line
    pub fn seek(&mut self, pos: SeekFrom<U>) {
        // FIXME remove magic number
        let mut cmd = 0b0100_0000;

        let (start, addr) = match pos {
            SeekFrom::Home(bytes) => (U::FIRST_LINE_ADDRESS.into(), bytes.into()),
            SeekFrom::Current(bytes) => (self.cursor_address, bytes.into()),
            SeekFrom::Line { line, bytes } => (line.into(), bytes.into()),
        };

        self.cursor_address = start + addr;

        cmd |= u8::from(self.cursor_address);

        self.connection.send(WriteMode::Command(cmd));
    }

    pub fn seek_cgram(self, address: u8) -> Display<P, U, DdRam> {
        let mut ddram_display = Display {
            connection: self.connection,
            cursor_address: Address::from(0),
            _ram_type: PhantomData::<DdRam>,
            _line_marker: PhantomData,
        };

        ddram_display.seek(SeekFrom::Home(address));

        ddram_display
    }
}
