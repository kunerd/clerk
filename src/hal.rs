pub enum WriteMode {
    Command(u8),
    Data(u8),
}

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

pub trait Init {
    fn init(&self);
}

pub trait Send {
    fn send(&self, mode: WriteMode);
}

pub trait Recieve {
    fn recieve(&self, mode: ReadMode) -> u8;
}

pub trait SendRaw {
    fn send_byte(&self, byte: u8);
}

pub trait RecieveRaw {
    fn receive_byte(&self) -> u8;
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

fn get_bit(val: u8, bit: u8) -> Level {
    if val & bit == bit {
        Level::High
    } else {
        Level::Low
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
