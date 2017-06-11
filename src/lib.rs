use std::thread::sleep;
use std::time::Duration;
use std::iter::Iterator;

pub enum Line {
    One,
    Two,
}

enum LcdMode {
    Command,
    Data,
}

pub struct LcdPins {
    pub register_select: u64,
    pub enable: u64,
    pub data4: u64,
    pub data5: u64,
    pub data6: u64,
    pub data7: u64,
}

static E_DELAY: u32 = 5;
const LCD_WIDTH: usize = 16;
const LCD_LINE_1: u8 = 0x80;
const LCD_LINE_2: u8 = 0xC0;

pub struct Lcd<T: LcdHardwareLayer + LcdHardwareLayerCleanup> {
    register_select: T,
    enable: T,
    data4: T,
    data5: T,
    data6: T,
    data7: T,
}

enum LogicLevels {
    High,
    Low,
}

pub trait LcdHardwareLayer {
    fn init(&self) where Self: LcdHardwareLayerCleanup {}
    // fn cleanup(&self) {}

    // fn set_value(level: LogicLevels) {}
    fn set_value(&self, u8) -> Result<(), ()>;
}

pub trait LcdHardwareLayerCleanup {
    fn cleanup(&self) {}
}

impl<T: From<u64> + LcdHardwareLayer + LcdHardwareLayerCleanup> Lcd<T> {
    pub fn from_pin(pins: LcdPins) -> Lcd<T> {
        let lcd = Lcd {
            register_select: T::from(pins.register_select),
            enable: T::from(pins.enable),
            data4: T::from(pins.data4),
            data5: T::from(pins.data5),
            data6: T::from(pins.data6),
            data7: T::from(pins.data7),
        };

        lcd.register_select.init();
        lcd.enable.init();
        lcd.data4.init();
        lcd.data5.init();
        lcd.data6.init();
        lcd.data7.init();

        lcd.send_byte(0x33, LcdMode::Command);
        lcd.send_byte(0x32, LcdMode::Command);
        lcd.send_byte(0x28, LcdMode::Command);
        lcd.send_byte(0x0C, LcdMode::Command);
        lcd.send_byte(0x06, LcdMode::Command);
        lcd.send_byte(0x01, LcdMode::Command);

        lcd
    }

    pub fn set_line(&self, line: Line) {
        let line = match line {
            Line::One => LCD_LINE_1,
            Line::Two => LCD_LINE_2,
        };

        self.send_byte(line, LcdMode::Command);
    }

    fn send_byte(&self, value: u8, mode: LcdMode) {
        let wait_time = Duration::new(0, E_DELAY);

        match mode {
                LcdMode::Data => self.register_select.set_value(1),
                LcdMode::Command => self.register_select.set_value(0),
            }
            .unwrap();

        self.data4.set_value(0).unwrap();
        self.data5.set_value(0).unwrap();
        self.data6.set_value(0).unwrap();
        self.data7.set_value(0).unwrap();

        if value & 0x10 == 0x10 {
            self.data4.set_value(1).unwrap();
        }
        if value & 0x20 == 0x20 {
            self.data5.set_value(1).unwrap();
        }
        if value & 0x40 == 0x40 {
            self.data6.set_value(1).unwrap();
        }
        if value & 0x80 == 0x80 {
            self.data7.set_value(1).unwrap();
        }

        sleep(wait_time);
        self.enable.set_value(1).unwrap();
        sleep(wait_time);
        self.enable.set_value(0).unwrap();
        sleep(wait_time);

        self.data4.set_value(0).unwrap();
        self.data5.set_value(0).unwrap();
        self.data6.set_value(0).unwrap();
        self.data7.set_value(0).unwrap();

        if value & 0x01 == 0x01 {
            self.data4.set_value(1).unwrap();
        }
        if value & 0x02 == 0x02 {
            self.data5.set_value(1).unwrap();
        }
        if value & 0x04 == 0x04 {
            self.data6.set_value(1).unwrap();
        }
        if value & 0x08 == 0x08 {
            self.data7.set_value(1).unwrap();
        }

        sleep(wait_time);
        self.enable.set_value(1).unwrap();
        sleep(wait_time);
        self.enable.set_value(0).unwrap();
        sleep(wait_time);
    }

    pub fn send_message(&self, msg: &str) {
        for c in msg.as_bytes().iter().take(LCD_WIDTH) {
            self.send_byte(*c, LcdMode::Data);
        }
    }
}

impl<T: LcdHardwareLayer + LcdHardwareLayerCleanup> Drop for Lcd<T> {
    fn drop(&mut self) {
        self.register_select.cleanup();
        self.enable.cleanup();
        self.data4.cleanup();
        self.data5.cleanup();
        self.data6.cleanup();
        self.data7.cleanup();
    }
}
