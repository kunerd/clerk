extern crate clerk;
extern crate sysfs_gpio;

use clerk::{CursorBlinking, CursorState, DataPins4Lines, DefaultLines, Display,
            DisplayControlBuilder, DisplayState, FunctionSetBuilder, Pins, SeekFrom};

mod utils;
use utils::ExternPin;
use utils::CustomDelay;

fn main() {
    let pins = Pins {
        register_select: ExternPin::new(2),
        read: ExternPin::new(3),
        enable: ExternPin::new(4),
        data: DataPins4Lines {
            data4: ExternPin::new(16),
            data5: ExternPin::new(19),
            data6: ExternPin::new(26),
            data7: ExternPin::new(20),
        },
    };

    let mut lcd: Display<_, DefaultLines> = Display::from_pins(pins);

    lcd.init(&FunctionSetBuilder::default());

    lcd.set_display_control(
        DisplayControlBuilder::default()
            .set_display(DisplayState::On)
            .set_cursor(CursorState::Off)
            .set_cursor_blinking(CursorBlinking::On),
    );

    lcd.write_message("Hello");

    lcd.seek(SeekFrom::Home(0));
    let value = lcd.read_byte();
    println!("Value is: {}", value as char);

    let (busy_flag, address) = lcd.read_busy_flag();
    println!("Busy Flag: {}, Address: {}", busy_flag, address);

    let value = lcd.read_byte();
    println!("Value is: {}", value as char);

    let (busy_flag, address) = lcd.read_busy_flag();
    println!("Busy Flag: {}, Address: {}", busy_flag, address);

    lcd.seek(SeekFrom::Current(0));
    lcd.write_message("llo World!");
}
