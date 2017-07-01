bitflags! {
    struct EntryModeFlags: u8 {
        const ENTRY_MODE            = 0b00000100;
        const CURSOR_MOVE_INCREMENT = 0b00000010;
        const DISPLAY_SHIFT_ON      = 0b00000001;
        const CURSOR_MOVE_DECREMENT = 0b00000000;
        const DISPLAY_SHIFT_OFF     = 0b00000000;
    }
}

/// Enumeration of possible methods to move.
#[derive(Clone, Copy)]
pub enum MoveDirection {
    /// Moves right.
    Increment,
    /// Moves left.
    Decrement,
}

impl From<MoveDirection> for EntryModeFlags {
    fn from(direction: MoveDirection) -> Self {
        match direction {
            MoveDirection::Increment => CURSOR_MOVE_INCREMENT,
            MoveDirection::Decrement => CURSOR_MOVE_DECREMENT,
        }
    }
}

/// Enumeration to set display shift.
#[derive(Clone, Copy)]
pub enum DisplayShift {
    On,
    Off,
}

impl From<DisplayShift> for EntryModeFlags {
    fn from(shift: DisplayShift) -> Self {
        match shift {
            DisplayShift::On => DISPLAY_SHIFT_ON,
            DisplayShift::Off => DISPLAY_SHIFT_OFF,
        }
    }
}

/// A struct for creating display entry mode settings.
pub struct EntryModeBuilder {
    move_direction: MoveDirection,
    display_shift: DisplayShift,
}

impl EntryModeBuilder {
    /// Make a new `EntryModeBuilder` with the default settings described below.
    ///
    /// The default settings are:
    ///
    ///  - **move direction:**
    ///     - `Increment`
    ///  - **display_shift:**
    ///     - `Off`
    pub fn new() -> EntryModeBuilder {
        EntryModeBuilder {
            move_direction: MoveDirection::Increment,
            display_shift: DisplayShift::Off,
        }
    }

    /// Sets the direction the read/write cursor is moved when a character code is written to or
    /// read from the display.
    pub fn set_move_direction(&mut self, direction: MoveDirection) -> &mut EntryModeBuilder {
        self.move_direction = direction;
        self
    }

    /// Sets the display shift, which will be performed on character write, either `On` or `Off`.
    ///
    /// If display shift is enabled, it will seem as if the cursor does not move but the display
    /// does.
    ///
    /// **Note:** The display does not shift when reading.
    pub fn set_display_shift(&mut self, shift: DisplayShift) -> &mut EntryModeBuilder {
        self.display_shift = shift;
        self
    }

    pub fn build_command(&self) -> u8 {
        let mut cmd = ENTRY_MODE;

        cmd |= EntryModeFlags::from(self.move_direction);
        cmd |= EntryModeFlags::from(self.display_shift);

        cmd.bits()
    }
}

impl Default for EntryModeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
