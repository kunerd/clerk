// TODO: make FIRST_LINE_ADDRESS configurable via trait constant
pub const FIRST_LINE_ADDRESS: u8 = 0x00;
const SECOND_LINE_ADDRESS: u8 = 0x40;

/// Enumeration of default lines.
pub enum DefaultLines {
    One,
    Two,
}

impl From<DefaultLines> for u8 {
    /// Returns the hardware address of the line.
    fn from(line: DefaultLines) -> Self {
        match line {
            DefaultLines::One => FIRST_LINE_ADDRESS,
            DefaultLines::Two => SECOND_LINE_ADDRESS,
        }
    }
}
