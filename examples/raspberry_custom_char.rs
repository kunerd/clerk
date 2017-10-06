use std::str;

extern crate clerk;
extern crate sysfs_gpio;

use clerk::{CursorBlinking, CursorState, DefaultLines, Display, DisplayPins, DisplayState,
            SeekFrom};

mod utils;
use utils::ExternPin;
use utils::CustomDelay;

fn main() {
    let pins = DisplayPins {
        register_select: 2,
        read: 3,
        enable: 4,
        data4: 16,
        data5: 19,
        data6: 26,
        data7: 20,
    };

    let mut lcd: Display<ExternPin, DefaultLines, CustomDelay> = Display::from_pins(pins);

    lcd.set_display_control(|e| {
        e.set_display(DisplayState::On)
            .set_cursor(CursorState::Off)
            .set_cursor_blinking(CursorBlinking::On);
    });

    lcd.seek_cgram(SeekFrom::Home(0));
    let character = [
        0b0_1110,
        0b1_0101,
        0b1_1111,
        0b1_0101,
        0b0_1110,
        0b0_0100,
        0b0_0100,
        0b1_1111,
    ];
    lcd.write_message(str::from_utf8(&character).unwrap());

    lcd.seek_cgram(SeekFrom::Home(0));
    println!("Created custom char is: ");
    for _ in 0..8 {
        let value = lcd.read_byte();
        println!("{:#08b}", value)
    }

    lcd.seek(SeekFrom::Home(0));
    lcd.write(0);
}
