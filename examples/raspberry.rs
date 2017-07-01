extern crate sysfs_gpio;
extern crate clerk;

use clerk::{SeekFrom, Display, DisplayPins, DisplayHardwareLayer, ShiftTo, DefaultLines,
            DisplayState, CursorState, CursorBlinking};

use sysfs_gpio::{Direction, Pin};

struct ExternPin(Pin);

impl From<u64> for ExternPin {
    fn from(i: u64) -> Self {
        ExternPin(Pin::new(i))
    }
}

impl DisplayHardwareLayer for ExternPin {
    fn init(&self) {
        self.0.export().unwrap();
        self.0.set_direction(Direction::Out).unwrap();
    }

    fn cleanup(&self) {
        self.0.unexport().unwrap();
    }

    fn set_value(&self, value: u8) -> Result<(), ()> {
        self.0.set_value(value).map_err(|_| ())
    }
}

fn main() {
    let pins = DisplayPins {
        register_select: 2,
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


    lcd.seek(SeekFrom::Line {
        line: DefaultLines::One,
        bytes: 0,
    });
    lcd.shift_cursor(ShiftTo::Right(2));
    lcd.write_message("Hallo");

    lcd.seek(SeekFrom::Line {
        line: DefaultLines::Two,
        bytes: 0,
    });
    lcd.shift_cursor(ShiftTo::Right(2));
    lcd.write_message("du");
    lcd.shift_cursor(ShiftTo::Left(2));

    lcd.shift(ShiftTo::Right(4));

    lcd.seek(SeekFrom::Current(5));
}
