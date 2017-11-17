bitflags! {
    struct EntryModeFlags: u8 {
        const ENTRY_MODE            = 0b0000_0100;
        const CURSOR_MOVE_INCREMENT = 0b0000_0010;
        const DISPLAY_SHIFT_ON      = 0b0000_0001;
        const CURSOR_MOVE_DECREMENT = 0b0000_0000;
        const DISPLAY_SHIFT_OFF     = 0b0000_0000;
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
            MoveDirection::Increment => EntryModeFlags::CURSOR_MOVE_INCREMENT,
            MoveDirection::Decrement => EntryModeFlags::CURSOR_MOVE_DECREMENT,
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
            DisplayShift::On => EntryModeFlags::DISPLAY_SHIFT_ON,
            DisplayShift::Off => EntryModeFlags::DISPLAY_SHIFT_OFF,
        }
    }
}

/// A struct for creating display entry mode settings.
pub struct EntryModeBuilder {
    move_direction: MoveDirection,
    display_shift: DisplayShift,
}

impl EntryModeBuilder {
    /// Sets the direction the read/write cursor is moved when a character code is written to or
    /// read from the display.
    pub fn set_move_direction(&mut self, direction: MoveDirection) -> &mut Self {
        self.move_direction = direction;
        self
    }

    /// Sets the display shift, which will be performed on character write, either `On` or `Off`.
    ///
    /// If display shift is enabled, it will seem as if the cursor does not move but the display
    /// does.
    ///
    /// **Note:** The display does not shift when reading.
    pub fn set_display_shift(&mut self, shift: DisplayShift) -> &mut Self {
        self.display_shift = shift;
        self
    }

    pub(crate) fn build_command(&self) -> u8 {
        let mut cmd = EntryModeFlags::ENTRY_MODE;

        cmd |= EntryModeFlags::from(self.move_direction);
        cmd |= EntryModeFlags::from(self.display_shift);

        cmd.bits()
    }
}

impl Default for EntryModeBuilder {
    /// Make a new `EntryModeBuilder` with the default settings described below.
    ///
    /// The default settings are:
    ///
    ///  - **move direction:**
    ///     - `Increment`
    ///  - **display_shift:**
    ///     - `Off`
    fn default() -> Self {
        Self {
            move_direction: MoveDirection::Increment,
            display_shift: DisplayShift::Off,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY_MODE_FLAG: u8 = 0b0000_0100;
    const MOVE_DIRECTION_FLAG: u8 = 0b0000_0010;
    const DISPLAY_SHIFT_FLAG: u8 = 0b0000_0001;

    fn has_bit(value: u8, bitmask: u8) -> bool {
        value & bitmask == bitmask
    }

    #[test]
    fn entry_mode_flag() {
        let b = EntryModeBuilder::default();
        let cmd = b.build_command();

        assert!(has_bit(cmd, ENTRY_MODE_FLAG));
    }

    #[test]
    fn default_move_direction() {
        let b = EntryModeBuilder::default();
        let cmd = b.build_command();

        assert!(has_bit(cmd, MOVE_DIRECTION_FLAG));
    }

    #[test]
    fn set_move_direction() {
        let mut b = EntryModeBuilder::default();

        let cmd = b.build_command();
        assert!(has_bit(cmd, MOVE_DIRECTION_FLAG));

        b.set_move_direction(MoveDirection::Decrement);

        let cmd = b.build_command();
        assert_eq!(has_bit(cmd, MOVE_DIRECTION_FLAG), false);
    }

    #[test]
    fn default_display_shift() {
        let b = EntryModeBuilder::default();
        let cmd = b.build_command();

        assert_eq!(has_bit(cmd, DISPLAY_SHIFT_FLAG), false);
    }

    #[test]
    fn set_display_shift() {
        let mut b = EntryModeBuilder::default();

        let cmd = b.build_command();
        assert_eq!(has_bit(cmd, DISPLAY_SHIFT_FLAG), false);

        b.set_display_shift(DisplayShift::On);

        let cmd = b.build_command();
        assert!(has_bit(cmd, DISPLAY_SHIFT_FLAG));
    }
}
