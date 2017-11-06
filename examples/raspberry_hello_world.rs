extern crate clerk;
extern crate sysfs_gpio;

use clerk::{CursorBlinking, CursorState, DefaultLines, Display, DisplayControlBuilder,
            DisplayPins, DisplayState, FunctionSetBuilder, LineNumber, SeekFrom};

mod utils;
use utils::ExternPin;
use utils::CustomDelay;

fn main() {
    let pins = DisplayPins {
        register_select: ExternPin::new(2),
        read: ExternPin::new(3),
        enable: ExternPin::new(4),
        data4: ExternPin::new(16),
        data5: ExternPin::new(19),
        data6: ExternPin::new(26),
        data7: ExternPin::new(20),
    };

    let mut lcd: Display<ExternPin, DefaultLines, CustomDelay> = Display::from_pins(pins);

    lcd.init(FunctionSetBuilder::default().set_line_number(LineNumber::Two));

    lcd.set_display_control(
        DisplayControlBuilder::default()
            .set_display(DisplayState::On)
            .set_cursor(CursorState::Off)
            .set_cursor_blinking(CursorBlinking::On),
    );

    lcd.write_message("Hello");

    lcd.seek(SeekFrom::Line {
        line: DefaultLines::Two,
        bytes: 5,
    });

    lcd.write_message("world!");
}
