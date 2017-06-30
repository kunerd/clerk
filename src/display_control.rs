use super::DISPLAY_CONTROL;

bitflags! {
    struct DisplayControl: u8 {
        // FIXME refactor same values
        const DISPLAY_OFF           = 0b00000000;
        const CURSOR_OFF            = 0b00000000;
        const CURSOR_BLINKING_OFF   = 0b00000000;
        const DISPLAY_ON            = 0b00000100;
        const CURSOR_ON             = 0b00000010;
        const CURSOR_BLINKING_ON    = 0b00000001;
    }
}

#[derive(Default)]
/// A struct for creating display control settings.
pub struct DisplayControlBuilder {
    // FIXME use enum instead of bool
    display: bool,
    cursor: bool,
    blink: bool,
}

impl DisplayControlBuilder {
    /// Makes a new `DisplayControlBuilder` using the default settings described below.
    ///
    /// The default settings are:
    ///
    ///  - **display:**
    ///     - `On`
    ///  - **cursor:**
    ///     - `Off`
    ///  - **blinking of cursor:**
    ///     - `Off`
    pub fn new() -> DisplayControlBuilder {
        DisplayControlBuilder {
            display: true,
            cursor: false,
            blink: false,
        }
    }

    /// Sets the entire display `On` or `Off`.
    ///
    /// Default is `On`.
    pub fn set_display(&mut self, status: bool) -> &mut DisplayControlBuilder {
        self.display = status;
        self
    }

    /// Sets the cursor `On` or `Off`.
    ///
    /// Default is `Off`.
    ///
    /// **Note:** This will not change cursor move direction or any other settings.
    pub fn set_cursor(&mut self, cursor: bool) -> &mut DisplayControlBuilder {
        self.cursor = cursor;
        self
    }

    /// Sets the blinking of the cursor `On` of `Off`.
    ///
    /// Default is `Off`.
    pub fn set_cursor_blinking(&mut self, blink: bool) -> &mut DisplayControlBuilder {
        self.blink = blink;
        self
    }

    pub fn build_command(&self) -> u8 {
        let mut cmd = DISPLAY_CONTROL.bits();

        cmd |= if self.display {
            DISPLAY_ON.bits()
        } else {
            DISPLAY_OFF.bits()
        };

        cmd |= if self.cursor {
            CURSOR_ON.bits()
        } else {
            CURSOR_OFF.bits()
        };

        cmd |= if self.cursor {
            CURSOR_BLINKING_ON.bits()
        } else {
            CURSOR_BLINKING_OFF.bits()
        };

        cmd
    }
}
