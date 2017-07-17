extern crate sysfs_gpio;
extern crate clerk;

use clerk::{SeekFrom, Display, DisplayPins, DisplayHardwareLayer, DefaultLines, DisplayState,
            CursorState, CursorBlinking, Direction};

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

    fn set_value(&self, value: u8) -> Result<(), ()> {
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

    lcd.write_message("Hello");

    lcd.seek(SeekFrom::Line {
        line: DefaultLines::Two,
        bytes: 5,
    });

    lcd.write_message("world!");
}
