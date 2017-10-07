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
    lcd.init(|_| {});

    lcd.set_display_control(|e| {
        e.set_display(DisplayState::On)
            .set_cursor(CursorState::Off)
            .set_cursor_blinking(CursorBlinking::On);
    });

    lcd.write_message("Hello");

    lcd.seek(SeekFrom::Home(0));
    let value = lcd.read_byte();
    println!("Value is: {}", value as char);

    let value = lcd.read_byte();
    println!("Value is: {}", value as char);

    let (busy_flag, address) = lcd.read_busy_flag();
    println!("Busy Flag: {}, Address: {}", busy_flag, address);

    let (busy_flag, address) = lcd.read_busy_flag();
    println!("Busy Flag: {}, Address: {}", busy_flag, address);

    lcd.seek(SeekFrom::Current(0));
    lcd.write_message("llo World!");
}
