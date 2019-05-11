use address::{Address, Overflow};

const SECOND_LINE_ADDRESS: u8 = 0x40;

/// This trait is used to specify the start address of the display data RAM.
pub trait Home {
    const FIRST_LINE_ADDRESS: u8 = 0x00;
}

/// Enumeration of default lines.
pub enum DefaultLines {
    One,
    Two,
}

impl Home for DefaultLines {}

impl<T: Overflow> From<DefaultLines> for Address<T> {
    /// Returns the hardware address of the line.
    fn from(line: DefaultLines) -> Self {
        let raw_addr = match line {
            DefaultLines::One => DefaultLines::FIRST_LINE_ADDRESS,
            DefaultLines::Two => SECOND_LINE_ADDRESS,
        };

        Address::from(raw_addr)
    }
}
