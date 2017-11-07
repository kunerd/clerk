const SECOND_LINE_ADDRESS: u8 = 0x40;

pub trait Home {
    const FIRST_LINE_ADDRESS: u8 = 0x00;
}

/// Enumeration of default lines.
pub enum DefaultLines {
    One,
    Two,
}

impl Home for DefaultLines {}

impl From<DefaultLines> for u8 {
    /// Returns the hardware address of the line.
    fn from(line: DefaultLines) -> Self {
        match line {
            DefaultLines::One => DefaultLines::FIRST_LINE_ADDRESS,
            DefaultLines::Two => SECOND_LINE_ADDRESS,
        }
    }
}
