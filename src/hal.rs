/// Enumeration possible write operations.
#[derive(Debug, PartialEq)]
pub enum WriteMode {
    Command(u8),
    Data(u8),
}

/// Enumeration possible read operations.
pub enum ReadMode {
    Data,
    BusyFlag,
}

/// Enumeration of possible data directions of a pin.
#[derive(Debug, PartialEq)]
pub enum Direction {
    In,
    Out,
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

/// This trait is used to provide an initialization implementation for a [`Display`] connection.
///
/// [`Display`]: struct.Display.html
pub trait Init {
    /// Initializes the connection.
    fn init<D: Delay>(&mut self, delay: &mut D);
}

/// This trait is used to provide an implementation for sending data via a [`Display`] connection.
///
/// [`Display`]: struct.Display.html
pub trait Send {
    /// Sends data via the connection.
    fn send<D: Delay>(&mut self, mode: WriteMode, delay: &mut D);
}

/// This trait is used to provide an implementation for receiving data via a [`Display`] connection.
///
/// [`Display`]: struct.Display.html
pub trait Receive {
    fn receive<D: Delay>(&mut self, mode: ReadMode, delay: &mut D) -> u8;
}

pub trait SendRaw {
    fn send_byte<D: Delay>(&mut self, byte: u8, delay: &mut D);
}

pub trait ReceiveRaw {
    fn receive_byte<D: Delay>(&mut self, delay: &mut D) -> u8;
}

/// The `DisplayHardwareLayer` trait is intended to be implemented by the library user as a thin
/// wrapper around the hardware specific system calls.
pub trait DisplayHardwareLayer {
    /// Initializes an I/O pin.
    fn init(&mut self) {}
    /// Cleanup an I/O pin.
    fn cleanup(&mut self) {}

    fn set_direction(&mut self, direction: Direction);
    /// Sets a value on an I/O pin.
    fn set_level(&mut self, level: Level);

    fn get_value(&mut self) -> u8;
}

/// The `Delay` trait is used to adapt the timing to the specific hardware and must be implemented
/// by the libary user.
pub trait Delay {
    /// The time (ns) between register select (RS) and read/write (R/W) to enable signal (E).
    const ADDRESS_SETUP_TIME: u8 = 0;
    /// The duration (ns) the enable signal is set to `High`.
    const ENABLE_PULSE_WIDTH: u8 = 0;
    /// The duration (ns) the data pins will be set after the enable signal was dropped.
    const DATA_HOLD_TIME: u8 = 0;

    /// The maximum execution time of instruction commands.
    const COMMAND_EXECUTION_TIME: u16 = 500;

    /// Wait for the given amount of nanoseconds.
    fn delay_ns(&mut self, _ns: u8) {}

    /// Wait for the given amount of microseconds.
    fn delay_us(&mut self, us: u16);
}

/// This struct is used for easily setting up [`ParallelConnection`]s.
///
/// [`ParallelConnection`]: struct.ParallelConnection.html
pub struct Pins<RS, R, E, D> {
    pub register_select: RS,
    pub read: R,
    pub enable: E,
    pub data: D,
}

impl<RS, R, E, D> Pins<RS, R, E, D> {
    /// Converts the pin setup into a [`ParallelConnection`] that is by `Display` to communicate
    /// with the LCD device.
    ///
    /// [`ParallelConnection`]: struct.ParallelConnection.html
    pub fn into_connection<T>(self) -> ParallelConnection<RS, R, E, D> {
        ParallelConnection {
            register_select: self.register_select,
            read: self.read,
            enable: self.enable,
            data: self.data,
        }
    }
}

/// The parallel connection mode is the most common wiring mode for HD44780 compliant displays.
/// It can be used with either four ([`DataPins4Lines`]) or eight ([`DataPins8Lines`]) data lines.
///
/// [`DataPins4Lines`]: struct.DataPins4Lines.html
/// [`DataPins8Lines`]: struct.DataPins8Lines.html
pub struct ParallelConnection<RS, R, E, D> {
    register_select: RS,
    read: R,
    enable: E,
    data: D,
}

impl<RS, R, E, P4, P5, P6, P7> Init
    for ParallelConnection<RS, R, E, DataPins4Lines<P4, P5, P6, P7>>
where
    RS: DisplayHardwareLayer,
    R: DisplayHardwareLayer,
    E: DisplayHardwareLayer,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
    DataPins4Lines<P4, P5, P6, P7>: Init,
{
    fn init<D: Delay>(&mut self, delay: &mut D) {
        self.register_select.init();
        self.register_select.set_direction(Direction::Out);

        self.read.init();
        self.read.set_direction(Direction::Out);

        self.enable.init();
        self.enable.set_direction(Direction::Out);

        self.data.init(delay);

        self.register_select.set_level(Level::Low);
        self.read.set_level(Level::Low);
        self.enable.set_level(Level::Low);

        delay.delay_us(40_000);
        write_4bit(self, Nibble::Lower(0x03), delay);
        delay.delay_us(5000);
        write_4bit(self, Nibble::Lower(0x03), delay);
        delay.delay_us(100);
        write_4bit(self, Nibble::Lower(0x03), delay);
        delay.delay_us(100);
        write_4bit(self, Nibble::Lower(0x02), delay);
        delay.delay_us(100);
    }
}

impl<RS, R, E, D> Send for ParallelConnection<RS, R, E, D>
where
    Self: SendRaw,
    RS: DisplayHardwareLayer,
    R: DisplayHardwareLayer,
{
    fn send<T: Delay>(&mut self, mode: WriteMode, delay: &mut T) {
        self.read.set_level(Level::Low);

        let (level, value) = mode.into();
        self.register_select.set_level(level);

        self.send_byte(value, delay);

        delay.delay_us(T::COMMAND_EXECUTION_TIME);
    }
}

impl<RS, R, E, D> Receive for ParallelConnection<RS, R, E, D>
where
    Self: ReceiveRaw,
    RS: DisplayHardwareLayer,
    R: DisplayHardwareLayer,
{
    fn receive<T: Delay> (&mut self, mode: ReadMode, delay: &mut T) -> u8 {
        self.read.set_level(Level::High);

        match mode {
            ReadMode::Data => self.register_select.set_level(Level::High),
            ReadMode::BusyFlag => self.register_select.set_level(Level::Low),
        };

        self.receive_byte(delay)
    }
}

// FIXME: WARNING - dummy implementation, not tested
impl<RS, R, E, P0, P1, P2, P3, P4, P5, P6, P7> SendRaw
    for ParallelConnection<RS, R, E, DataPins8Lines<P0, P1, P2, P3, P4, P5, P6, P7>>
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
    fn send_byte<T: Delay> (&mut self, byte: u8, delay: &mut T) {
        delay.delay_ns(T::ADDRESS_SETUP_TIME);
        self.enable.set_level(Level::High);

        self.data.data0.set_level(get_bit(byte, 0b0000_0001));
        self.data.data1.set_level(get_bit(byte, 0b0000_0010));
        self.data.data2.set_level(get_bit(byte, 0b0000_0100));
        self.data.data3.set_level(get_bit(byte, 0b0000_1000));
        self.data.data4.set_level(get_bit(byte, 0b0001_0000));
        self.data.data5.set_level(get_bit(byte, 0b0010_0000));
        self.data.data6.set_level(get_bit(byte, 0b0100_0000));
        self.data.data7.set_level(get_bit(byte, 0b1000_0000));

        delay.delay_ns(T::ENABLE_PULSE_WIDTH);
        self.enable.set_level(Level::Low);
        delay.delay_ns(T::DATA_HOLD_TIME);
    }
}

fn get_bit(val: u8, bit: u8) -> Level {
    if val & bit == bit {
        Level::High
    } else {
        Level::Low
    }
}

/// Eight data lines pin wiring setup.
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
    fn init<D>(&mut self, _delay: &mut D) {
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

/// Four data lines pin wiring setup.
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
    fn init<D>(&mut self, _delay: &mut D) {
        self.data4.init();
        self.data5.init();
        self.data6.init();
        self.data7.init();
    }
}

impl<RS, R, E, P4, P5, P6, P7> SendRaw
    for ParallelConnection<RS, R, E, DataPins4Lines<P4, P5, P6, P7>>
where
    E: DisplayHardwareLayer,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    fn send_byte<T: Delay>(&mut self, byte: u8, delay: &mut T) {
        self.data.data4.set_direction(Direction::Out);
        self.data.data5.set_direction(Direction::Out);
        self.data.data6.set_direction(Direction::Out);
        self.data.data7.set_direction(Direction::Out);

        write_4bit(self, Nibble::Upper(byte), delay);
        write_4bit(self, Nibble::Lower(byte), delay);
    }
}

fn write_4bit<RS, R, E, T, P4, P5, P6, P7>(
    pins: &mut ParallelConnection<RS, R, E, DataPins4Lines<P4, P5, P6, P7>>,
    nibble: Nibble,
    delay: &mut T,
) where
    E: DisplayHardwareLayer,
    T: Delay,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{

    delay.delay_ns(T::ADDRESS_SETUP_TIME);
    pins.enable.set_level(Level::High);

    let value: u8 = nibble.into();
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

    delay.delay_ns(T::ENABLE_PULSE_WIDTH);
    pins.enable.set_level(Level::Low);
    delay.delay_ns(T::DATA_HOLD_TIME);
}

impl<RS, R, E, P4, P5, P6, P7> ReceiveRaw
    for ParallelConnection<RS, R, E, DataPins4Lines<P4, P5, P6, P7>>
where
    E: DisplayHardwareLayer,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    fn receive_byte<T: Delay> (&mut self, delay: &mut T) -> u8 {
        self.data.data4.set_direction(Direction::In);
        self.data.data5.set_direction(Direction::In);
        self.data.data6.set_direction(Direction::In);
        self.data.data7.set_direction(Direction::In);

        let upper = read_single_nibble(self, delay);
        let lower = read_single_nibble(self, delay);

        let mut result = upper << 4;
        result |= lower & 0x0f;

        result
    }
}

fn read_single_nibble<RS, R, E, T, P4, P5, P6, P7>(
    pins: &mut ParallelConnection<RS, R, E, DataPins4Lines<P4, P5, P6, P7>>,
    delay: &mut T
) -> u8
where
    E: DisplayHardwareLayer,
    T: Delay,
    P4: DisplayHardwareLayer,
    P5: DisplayHardwareLayer,
    P6: DisplayHardwareLayer,
    P7: DisplayHardwareLayer,
{
    let mut result = 0u8;

    delay.delay_ns(T::ADDRESS_SETUP_TIME);
    pins.enable.set_level(Level::High);

    result |= pins.data.data7.get_value() << 3;
    result |= pins.data.data6.get_value() << 2;
    result |= pins.data.data5.get_value() << 1;
    result |= pins.data.data4.get_value();

    delay.delay_ns(T::ENABLE_PULSE_WIDTH);
    pins.enable.set_level(Level::Low);
    delay.delay_ns(T::DATA_HOLD_TIME);

    result
}
