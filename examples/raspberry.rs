extern crate sysfs_gpio;
extern crate hd44780_driver;

use hd44780_driver::{Line, Lcd, LcdPins, LcdHardwareLayer};

use sysfs_gpio::{Direction, Pin};

struct ExternPin(Pin);

impl From<u64> for ExternPin {
    fn from(i: u64) -> Self {
        ExternPin(Pin::new(i))
    }
}

impl LcdHardwareLayer for ExternPin {
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
    let pins = LcdPins {
        register_select: 2,
        enable: 4,
        data4: 16,
        data5: 19,
        data6: 26,
        data7: 20,
    };

    let lcd: Lcd<ExternPin> = Lcd::from_pin(pins);

    lcd.set_display_control(|e| {
        e.set_display(true)
            .set_cursor(true)
            .set_cursor_blinking(true);
    });

    lcd.set_line(Line::One);
    lcd.send_message("Hallo");
}
