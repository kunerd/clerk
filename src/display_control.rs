bitflags! {
    struct DisplayControlFlags: u8 {
        const DISPLAY_CONTROL       = 0b00001000;
        const DISPLAY_ON            = 0b00000100;
        const CURSOR_ON             = 0b00000010;
        const CURSOR_BLINKING_ON    = 0b00000001;
        const DISPLAY_OFF           = 0b00000000;
        const CURSOR_OFF            = 0b00000000;
        const CURSOR_BLINKING_OFF   = 0b00000000;
    }
}

/// State of a display.
#[derive(Clone, Copy)]
pub enum DisplayState {
    On,
    Off,
}

impl From<DisplayState> for DisplayControlFlags {
    fn from(state: DisplayState) -> Self {
        match state {
            DisplayState::On => DISPLAY_ON,
            DisplayState::Off => DISPLAY_OFF,
        }
    }
}

/// State of a cursor.
#[derive(Clone, Copy)]
pub enum CursorState {
    On,
    Off,
}

impl From<CursorState> for DisplayControlFlags {
    fn from(state: CursorState) -> Self {
        match state {
            CursorState::On => CURSOR_ON,
            CursorState::Off => CURSOR_OFF,
        }
    }
}

/// Sets cursor blinking.
#[derive(Clone, Copy)]
pub enum CursorBlinking {
    On,
    Off,
}

impl From<CursorBlinking> for DisplayControlFlags {
    fn from(state: CursorBlinking) -> Self {
        match state {
            CursorBlinking::On => CURSOR_BLINKING_ON,
            CursorBlinking::Off => CURSOR_BLINKING_OFF,
        }
    }
}

/// A struct for creating display control settings.
pub struct DisplayControlBuilder {
    display: DisplayState,
    cursor: CursorState,
    blinking: CursorBlinking,
}

impl DisplayControlBuilder {
    /// Sets the entire display `On` or `Off`.
    ///
    /// Default is `On`.
    pub fn set_display(&mut self, state: DisplayState) -> &mut DisplayControlBuilder {
        self.display = state;
        self
    }

    /// Sets the cursor `On` or `Off`.
    ///
    /// Default is `Off`.
    ///
    /// **Note:** This will not change cursor move direction or any other settings.
    pub fn set_cursor(&mut self, state: CursorState) -> &mut DisplayControlBuilder {
        self.cursor = state;
        self
    }

    /// Sets the blinking of the cursor `On` of `Off`.
    ///
    /// Default is `Off`.
    pub fn set_cursor_blinking(&mut self, blinking: CursorBlinking) -> &mut DisplayControlBuilder {
        self.blinking = blinking;
        self
    }

    pub fn build_command(&self) -> u8 {
        let mut cmd = DISPLAY_CONTROL;

        cmd |= DisplayControlFlags::from(self.display);
        cmd |= DisplayControlFlags::from(self.cursor);
        cmd |= DisplayControlFlags::from(self.blinking);

        cmd.bits()
    }
}

impl Default for DisplayControlBuilder {
    /// Makes a new `DisplayControlBuilder` using the default settings described below.
    ///
    /// The default settings are:
    ///
    ///  - **display:**
    ///     - `On`
    ///  - **cursor:**
    ///     - `Off`
    ///  - **blinkinging of cursor:**
    ///     - `Off`
    fn default() -> Self {
        Self {
            display: DisplayState::On,
            cursor: CursorState::Off,
            blinking: CursorBlinking::Off,
        }
    }
}
