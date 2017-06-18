#[macro_use]
extern crate bitflags;

use std::thread::sleep;
use std::time::Duration;
use std::iter::Iterator;

pub enum Line {
    One,
    Two,
}

bitflags! {
    struct Instructions: u8 {
        const CLEAR_DISPLAY     = 0b00000001;
        const RETURN_HOME       = 0b00000010;
        const ENTRY_MODE        = 0b00000100;
        const DISPLAY_CONTROL   = 0b00001000;
        const SHIFT             = 0b00010000;
        const FUNCTION_SET      = 0b00100000;
    }
}

bitflags! {
    struct CursorMoveDirection: u8 {
        const CURSOR_MOVE_DECREMENT = 0b00000000;
        const CURSOR_MOVE_INCREMENT = 0b00000010;
    }
}

bitflags! {
    struct DisplayShift: u8 {
        const DISPLAY_SHIFT_DISABLE = 0b00000000;
        const DISPLAY_SHIFT_ENABLE  = 0b00000001;
    }
}

bitflags! {
    struct DisplayControl: u8 {
        const DISPLAY_OFF   = 0b00000000;
        const CURSOR_OFF    = 0b00000000;
        const CURSOR_ON     = 0b00000010;
        const DISPLAY_ON    = 0b00000100;
    }
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

pub struct Lcd<T: LcdHardwareLayer> {
    register_select: T,
    enable: T,
    data4: T,
    data5: T,
    data6: T,
    data7: T,
}

// need a way to let the user set up how levels are interpreted by the hardware
enum LogicLevels {
    High,
    Low,
}

pub trait LcdHardwareLayer {
    fn init(&self) {}
    fn cleanup(&self) {}

    // fn set_value(level: LogicLevels) {}
    fn set_value(&self, u8) -> Result<(), ()>;
}

pub enum MoveDirection {
    Increment,
    Decrement,
}

pub struct EntryModeBuilder<'a, T>
    where T: 'a + From<u64> + LcdHardwareLayer
{
    lcd: &'a Lcd<T>,
    move_direction: MoveDirection,
    display_shift: bool,
}

impl<'a, T> EntryModeBuilder<'a, T>
    where T: From<u64> + LcdHardwareLayer
{
    fn new(lcd: &'a Lcd<T>) -> EntryModeBuilder<T> {
        EntryModeBuilder {
            lcd,
            move_direction: MoveDirection::Increment,
            display_shift: false,
        }
    }

    pub fn set_move_direction(&mut self, direction: MoveDirection) -> &'a mut EntryModeBuilder<T> {
        self.move_direction = direction;
        self
    }

    pub fn set_display_shift(&mut self, shift: bool) -> &'a mut EntryModeBuilder<T> {
        self.display_shift = shift;
        self
    }

    pub fn run(&self) {
        let mut cmd = ENTRY_MODE.bits();

        cmd |= match self.move_direction {
            MoveDirection::Increment => CURSOR_MOVE_INCREMENT.bits(),
            MoveDirection::Decrement => CURSOR_MOVE_DECREMENT.bits(),
        };

        cmd |= match self.display_shift {
            true => DISPLAY_SHIFT_ENABLE.bits(),
            false => DISPLAY_SHIFT_DISABLE.bits(),
        };

        self.lcd.send_byte(cmd, LcdMode::Command);
    }
}

pub struct DisplayControlBuilder<'a, T>
    where T: 'a + From<u64> + LcdHardwareLayer
{
    lcd: &'a Lcd<T>,
    display: bool,
    cursor: bool,
}

impl<'a, T> DisplayControlBuilder<'a, T>
    where T: From<u64> + LcdHardwareLayer
{
    pub fn new(lcd: &'a Lcd<T>) -> DisplayControlBuilder<T> {
        DisplayControlBuilder {
            lcd,
            display: true,
            cursor: false,
        }
    }

    pub fn set_display(&mut self, status: bool) -> &'a mut DisplayControlBuilder<T> {
        self.display = status;
        self
    }

    pub fn set_cursor(&mut self, cursor: bool) -> &'a mut DisplayControlBuilder<T> {
        self.cursor = cursor;
        self
    }

    pub fn run(&self) {
        let mut cmd = DISPLAY_CONTROL.bits();

        cmd |= match self.display {
            true => DISPLAY_ON.bits(),
            false => DISPLAY_OFF.bits(),
        };

        cmd |= match self.cursor {
            true => CURSOR_ON.bits(),
            false => CURSOR_OFF.bits(),
        };

        println!("{}", cmd);

        self.lcd.send_byte(cmd, LcdMode::Command);
    }
}

impl<T: From<u64> + LcdHardwareLayer> Lcd<T> {
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

        // Initializing by Instruction
        lcd.send_byte(0x33, LcdMode::Command);
        lcd.send_byte(0x32, LcdMode::Command);
        // FuctionSet: Data length 4bit + 2 lines
        lcd.send_byte(0x28, LcdMode::Command);
        // DisplayControl: Display on, Cursor off + cursor blinking off
        lcd.send_byte(0x0C, LcdMode::Command);
        // EntryModeSet: Cursor move direction inc + no display shift
        lcd.send_byte(0x06, LcdMode::Command);
        lcd.clear(); // ClearDisplay

        lcd
    }

    pub fn set_entry_mode(&self) -> EntryModeBuilder<T> {
        EntryModeBuilder::new(self)
    }

    pub fn display_control(&self) -> DisplayControlBuilder<T> {
        DisplayControlBuilder::new(self)
    }

    pub fn clear(&self) {
        self.send_byte(CLEAR_DISPLAY.bits(), LcdMode::Command);
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

impl<T: LcdHardwareLayer> Drop for Lcd<T> {
    fn drop(&mut self) {
        self.register_select.cleanup();
        self.enable.cleanup();
        self.data4.cleanup();
        self.data5.cleanup();
        self.data6.cleanup();
        self.data7.cleanup();
    }
}
