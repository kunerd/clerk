bitflags! {
    struct FunctionSetFlags: u8 {
        const FUNCTION_SET                  = 0b0010_0000;
        const INTERFACE_DATA_LENGTH_8BIT    = 0b0001_0000;
        const DISPLAY_LINES_NUMBER_2        = 0b0000_1000;
        const CHARACTER_FONT_5_10_DOTS      = 0b0000_0100;
        const INTERFACE_DATA_LENGTH_4BIT    = 0;
        const DISPLAY_LINES_NUMBER_1        = 0;
        const CHARACTER_FONT_5_8_DOTS       = 0;
    }
}

/// Enumeration of possible interface data lengths.
#[derive(Clone, Copy)]
pub enum DataLength {
    /// 4-bit mode
    FourBit,
    /// 8-bit mode
    EightBit,
}

impl From<DataLength> for FunctionSetFlags {
    fn from(length: DataLength) -> Self {
        match length {
            DataLength::FourBit => INTERFACE_DATA_LENGTH_4BIT,
            DataLength::EightBit => INTERFACE_DATA_LENGTH_8BIT,
        }
    }
}

/// Enumeration to set display line number.
#[derive(Clone, Copy)]
pub enum LineNumber {
    One,
    Two,
}

impl From<LineNumber> for FunctionSetFlags {
    fn from(number: LineNumber) -> Self {
        match number {
            LineNumber::One => DISPLAY_LINES_NUMBER_1,
            LineNumber::Two => DISPLAY_LINES_NUMBER_2,
        }
    }
}

/// Enumeration to set display character font.
#[derive(Clone, Copy)]
pub enum CharacterFont {
    Dots5By10,
    Dots5By8,
}

impl From<CharacterFont> for FunctionSetFlags {
    fn from(font: CharacterFont) -> Self {
        match font {
            CharacterFont::Dots5By10 => CHARACTER_FONT_5_10_DOTS,
            CharacterFont::Dots5By8 => CHARACTER_FONT_5_8_DOTS,
        }
    }
}

/// A struct for creating display function settings.
pub struct FunctionSetBuilder {
    data_length: DataLength,
    line_number: LineNumber,
    character_font: CharacterFont,
}

impl FunctionSetBuilder {
    pub fn set_data_length(&mut self, data_length: DataLength) -> &mut Self {
        self.data_length = data_length;
        self
    }

    pub fn set_line_number(&mut self, line_number: LineNumber) -> &mut Self {
        self.line_number = line_number;
        self
    }

    pub fn set_character_font(&mut self, character_font: CharacterFont) -> &mut Self {
        self.character_font = character_font;
        self
    }

    pub(crate) fn build_command(&self) -> u8 {
        let mut cmd = FUNCTION_SET;

        cmd |= FunctionSetFlags::from(self.data_length);
        cmd |= FunctionSetFlags::from(self.line_number);
        cmd |= FunctionSetFlags::from(self.character_font);

        cmd.bits()
    }
}

impl Default for FunctionSetBuilder {
    /// Make a new `FunctionSetBuilder` with the default settings described below.
    ///
    /// The default settings are:
    ///
    ///  - **data_length:**
    ///     - `FourBit` - 4-bit mode
    ///  - **line_number:**
    ///     - `One` - one line mode
    ///  - **character_font:**
    ///     - `Dots5By8` - 5x8 dots character font
    fn default() -> Self {
        Self {
            data_length: DataLength::FourBit,
            line_number: LineNumber::One,
            character_font: CharacterFont::Dots5By8,
        }
    }
}
