bitflags! {
    struct DisplayControlFlags: u8 {
        const DISPLAY_CONTROL       = 0b0000_1000;
        const DISPLAY_ON            = 0b0000_0100;
        const CURSOR_ON             = 0b0000_0010;
        const CURSOR_BLINKING_ON    = 0b0000_0001;
        const DISPLAY_OFF           = 0b0000_0000;
        const CURSOR_OFF            = 0b0000_0000;
        const CURSOR_BLINKING_OFF   = 0b0000_0000;
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
    pub fn set_display(&mut self, state: DisplayState) -> &mut Self {
        self.display = state;
        self
    }

    /// Sets the cursor `On` or `Off`.
    ///
    /// Default is `Off`.
    ///
    /// **Note:** This will not change cursor move direction or any other settings.
    pub fn set_cursor(&mut self, state: CursorState) -> &mut Self {
        self.cursor = state;
        self
    }

    /// Sets the blinking of the cursor `On` of `Off`.
    ///
    /// Default is `Off`.
    pub fn set_cursor_blinking(&mut self, blinking: CursorBlinking) -> &mut Self {
        self.blinking = blinking;
        self
    }

    pub(crate) fn build_command(&self) -> u8 {
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

#[cfg(test)]
mod tests {
    use super::*;

    const DISPLAY_CONTROL_FLAG: u8 = 0b0000_1000;
    const DISPLAY_STATE_FLAG: u8 = 0b0000_0100;
    const CURSOR_STATE_FLAG: u8 = 0b0000_0010;
    const CURSOR_BLINKING_FLAG: u8 = 0b0000_0001;

    fn has_bit(value: u8, bitmask: u8) -> bool {
        value & bitmask == bitmask
    }

    #[test]
    fn display_control_flag() {
        let b = DisplayControlBuilder::default();
        let cmd = b.build_command();

        assert!(has_bit(cmd, DISPLAY_CONTROL_FLAG));
    }

    #[test]
    fn default_display_state() {
        let b = DisplayControlBuilder::default();
        let cmd = b.build_command();

        assert!(has_bit(cmd, DISPLAY_STATE_FLAG));
    }

    #[test]
    fn set_display() {
        let mut b = DisplayControlBuilder::default();

        b.set_display(DisplayState::On);
        let cmd = b.build_command();
        assert!(has_bit(cmd, DISPLAY_STATE_FLAG));

        b.set_display(DisplayState::Off);
        let cmd = b.build_command();
        assert_eq!(has_bit(cmd, DISPLAY_STATE_FLAG), false);
    }

    #[test]
    fn default_cursor_state() {
        let b = DisplayControlBuilder::default();
        let cmd = b.build_command();

        assert_eq!(has_bit(cmd, CURSOR_STATE_FLAG), false);
    }

    #[test]
    fn set_cursor() {
        let mut b = DisplayControlBuilder::default();

        b.set_cursor(CursorState::On);
        let cmd = b.build_command();
        assert!(has_bit(cmd, CURSOR_STATE_FLAG));

        b.set_cursor(CursorState::Off);
        let cmd = b.build_command();
        assert_eq!(has_bit(cmd, CURSOR_STATE_FLAG), false);
    }

    #[test]
    fn default_cursor_blinking() {
        let b = DisplayControlBuilder::default();
        let cmd = b.build_command();

        assert_eq!(has_bit(cmd, CURSOR_BLINKING_FLAG), false);
    }

    #[test]
    fn set_cursor_blinking() {
        let mut b = DisplayControlBuilder::default();

        b.set_cursor_blinking(CursorBlinking::On);
        let cmd = b.build_command();
        assert!(has_bit(cmd, CURSOR_BLINKING_FLAG));

        b.set_cursor_blinking(CursorBlinking::Off);
        let cmd = b.build_command();
        assert_eq!(has_bit(cmd, CURSOR_STATE_FLAG), false);
    }
}
