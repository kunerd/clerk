use super::ENTRY_MODE;

bitflags! {
    struct ShiftDirectionDirection: u8 {
        const CURSOR_MOVE_DECREMENT = 0b00000000;
        const CURSOR_MOVE_INCREMENT = 0b00000010;
    }
}

bitflags! {
    struct DisplayShift: u8 {
        const DISPLAY_SHIFT_DISABLE = 0b00000000;
        const DISPLAY_SHIFT_ENABLE  = 0b00000001;
    }
}

/// Enumeration of possible methods to move.
pub enum MoveDirection {
    /// Moves right.
    Increment,
    /// Moves left.
    Decrement,
}

/// A struct for creating display entry mode settings.
pub struct EntryModeBuilder {
    move_direction: MoveDirection,
    display_shift: bool,
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
            display_shift: false,
        }
    }

    /// Sets the direction the read/write cursor is moved when a character code is written to or
    /// read from the display.
    pub fn set_move_direction(&mut self, direction: MoveDirection) -> &mut EntryModeBuilder {
        self.move_direction = direction;
        self
    }

    /// Sets the display shift, which will be on character write, either `On` or `Off`.
    ///
    /// If display shift is enabled, it will seem as if the cursor does not move but the display
    /// does.
    ///
    /// **Note:** The display does not shift when reading.
    pub fn set_display_shift(&mut self, shift: bool) -> &mut EntryModeBuilder {
        self.display_shift = shift;
        self
    }

    pub fn build_command(&self) -> u8 {
        let mut cmd = ENTRY_MODE.bits();

        cmd |= match self.move_direction {
            MoveDirection::Increment => CURSOR_MOVE_INCREMENT.bits(),
            MoveDirection::Decrement => CURSOR_MOVE_DECREMENT.bits(),
        };

        cmd |= if self.display_shift {
            DISPLAY_SHIFT_ENABLE.bits()
        } else {
            DISPLAY_SHIFT_DISABLE.bits()
        };

        cmd
    }
}
