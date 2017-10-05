use std::str;

extern crate clerk;
extern crate sysfs_gpio;

use clerk::{CursorBlinking, CursorState, DefaultLines, Direction, Display, DisplayHardwareLayer,
            DisplayPins, DisplayState, Level, SeekFrom};

struct ExternPin(sysfs_gpio::Pin);

impl From<u64> for ExternPin {
    fn from(i: u64) -> Self {
        ExternPin(sysfs_gpio::Pin::new(i))
    }
}

impl DisplayHardwareLayer for ExternPin {
    fn init(&self) {
        self.0.export().unwrap();
        self.0.set_direction(sysfs_gpio::Direction::Out).unwrap();
    }

    fn cleanup(&self) {
        self.0.unexport().unwrap();
    }

    fn set_direction(&self, direction: Direction) {
        let native_direction = match direction {
            Direction::In => sysfs_gpio::Direction::In,
            Direction::Out => sysfs_gpio::Direction::Out,
        };

        self.0.set_direction(native_direction).unwrap();
    }

    fn set_level(&self, level: Level) -> Result<(), ()> {
        let value = match level {
            Level::High => 1,
            Level::Low => 0,
        };

        self.0.set_value(value).map_err(|_| ())
    }

    fn get_value(&self) -> u8 {
        self.0.get_value().unwrap()
    }
}

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

    let mut lcd: Display<ExternPin, DefaultLines> = Display::from_pins(pins);

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

    let msg = [0];
    lcd.seek(SeekFrom::Home(0));
    lcd.write_message(str::from_utf8(&msg).unwrap());
}
