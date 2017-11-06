extern crate clerk;
extern crate itertools;

use std::cell::RefCell;
use itertools::{multizip, Itertools};

use clerk::{DefaultLines, Delay, Direction, Display, DisplayControlBuilder, DisplayHardwareLayer,
            DisplayPins, EntryModeBuilder, FunctionSetBuilder, Level, SeekFrom, ShiftTo};

#[derive(Default)]
pub struct PinMock {
    levels: RefCell<Vec<Level>>,
    directions: RefCell<Vec<Direction>>,
}

impl DisplayHardwareLayer for PinMock {
    fn init(&self) {}

    fn cleanup(&self) {}

    fn set_direction(&self, direction: Direction) {
        let mut directions = self.directions.borrow_mut();

        directions.push(direction);
    }

    fn set_level(&self, level: Level) {
        let mut levels = self.levels.borrow_mut();

        levels.push(level);
    }

    fn get_value(&self) -> u8 {
        panic!("Not implemented.")
    }
}

pub struct CustomDelayMock;

impl Delay for CustomDelayMock {
    fn delay_ns(_: u16) {
        // mhh
    }
}

fn setup_display() -> Display<PinMock, DefaultLines, CustomDelayMock> {
    let pins = DisplayPins {
        register_select: PinMock::default(),
        read: PinMock::default(),
        enable: PinMock::default(),
        data4: PinMock::default(),
        data5: PinMock::default(),
        data6: PinMock::default(),
        data7: PinMock::default(),
    };

    Display::from_pins(pins)
}

#[test]
fn test_init() {
    let lcd = setup_display();

    lcd.init(&FunctionSetBuilder::default());

    let pins = lcd.get_pins();
    {
        // check initialization
        let actual = pins.register_select.directions.borrow();
        assert_eq!(actual[..], [Direction::Out]);

        let actual = pins.read.directions.borrow();
        assert_eq!(actual[..], [Direction::Out]);

        let actual = pins.enable.directions.borrow();
        assert_eq!(actual[..], [Direction::Out]);
    }

    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0x33));
    assert_eq!(outp[1], to_level_slice(0x32));
    assert_eq!(outp[2], to_level_slice(0x20));
    assert_eq!(outp[3], to_level_slice(0x01));
}

#[test]
fn test_set_entry_mode() {
    let lcd = setup_display();

    lcd.set_entry_mode(&EntryModeBuilder::default());

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b0000_0110));
}

#[test]
fn test_set_display_control() {
    let lcd = setup_display();

    lcd.set_display_control(&DisplayControlBuilder::default());

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b0000_1100));
}

#[test]
#[ignore]
// TODO: needs clarification what happens on real hardware
fn test_shift_cursor_left() {
    let mut lcd = setup_display();

    lcd.shift_cursor(ShiftTo::Left(1));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b0001_0000));
}

#[test]
fn test_shift_cursor_right() {
    let mut lcd = setup_display();

    lcd.shift_cursor(ShiftTo::Right(1));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b0001_0100));
}

#[test]
fn test_shift_left() {
    let lcd = setup_display();

    lcd.shift(ShiftTo::Left(1));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b0001_1000));
}

#[test]
fn test_shift_right() {
    let lcd = setup_display();

    lcd.shift(ShiftTo::Right(1));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b0001_1100));
}

#[test]
fn test_clear() {
    let lcd = setup_display();

    lcd.clear();

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0x01));
}

#[test]
fn test_seek_from_home() {
    let mut lcd = setup_display();

    lcd.seek(SeekFrom::Home(3));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b1000_0011));
}

#[test]
fn test_seek_from_current() {
    let mut lcd = setup_display();

    lcd.seek(SeekFrom::Home(2));
    lcd.seek(SeekFrom::Current(1));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b1000_0010));
    assert_eq!(outp[1], to_level_slice(0b1000_0011));
}

#[test]
fn test_seek_from_line() {
    let mut lcd = setup_display();

    lcd.seek(SeekFrom::Line {
        line: DefaultLines::Two,
        bytes: 3,
    });

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b1100_0011));
}

#[test]
fn test_seek_cgram_from_home() {
    let mut lcd = setup_display();

    lcd.seek_cgram(SeekFrom::Home(3));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b0100_0011));
}

#[test]
fn test_seek_cgram_from_current() {
    let mut lcd = setup_display();

    lcd.seek_cgram(SeekFrom::Home(2));
    lcd.seek_cgram(SeekFrom::Current(1));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b0100_0010));
    assert_eq!(outp[1], to_level_slice(0b0100_0011));
}

#[test]
#[ignore]
// TODO: needs clarification: line does not make sense here,  For 5×8 dots, eight character
// patterns can be written, and for 5×10 dots, four character patterns can be written
fn test_seek_cgram_from_line() {
    let mut lcd = setup_display();

    lcd.seek_cgram(SeekFrom::Home(2));
    lcd.seek_cgram(SeekFrom::Current(1));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);

    assert_eq!(outp[0], to_level_slice(0b0100_0010));
    assert_eq!(outp[1], to_level_slice(0b0100_0011));
}

#[test]
fn test_write() {
    let mut lcd = setup_display();

    lcd.write(123);

    let pins = lcd.get_pins();

    {
        let actual = pins.read.levels.borrow();
        assert_eq!(actual[..], [Level::Low]);

        let actual = pins.register_select.levels.borrow();
        assert_eq!(actual[..], [Level::High]);
    }

    let outp = flat_pins(pins);
    assert_eq!(outp[0], to_level_slice(123));
}

#[test]
fn test_write_updates_address_counter() {
    let mut lcd = setup_display();

    lcd.seek(SeekFrom::Home(0));
    lcd.write(12);
    lcd.write(34);
    lcd.seek(SeekFrom::Current(0));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);
    assert_eq!(outp[3], to_level_slice(0b1000_0010));
}

#[test]
fn test_write_message() {
    let mut lcd = setup_display();

    lcd.write_message("Hi");

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);
    assert_eq!(outp[0], to_level_slice(b'H'));
    assert_eq!(outp[1], to_level_slice(b'i'));
}

#[test]
fn test_write_message_increments_address_counter() {
    let mut lcd = setup_display();

    lcd.write_message("Hi");
    lcd.seek(SeekFrom::Current(0));

    let pins = lcd.get_pins();
    let outp = flat_pins(pins);
    assert_eq!(outp[2], to_level_slice(0b1000_0010));
}

fn flat_pins(pins: DisplayPins<PinMock>) -> Vec<Vec<Level>> {
    let mut r = vec![];

    let data4 = pins.data4.levels.into_inner();
    let data5 = pins.data5.levels.into_inner();
    let data6 = pins.data6.levels.into_inner();
    let data7 = pins.data7.levels.into_inner();

    for (d4, d5, d6, d7) in multizip((data4, data5, data6, data7)) {
        r.push(vec![d7, d6, d5, d4]);
    }

    r.into_iter()
        .chunks(2)
        .into_iter()
        .map(|u| u.into_iter().flat_map(|v| v).collect::<Vec<_>>())
        .collect::<Vec<_>>()
}

fn to_level_slice(v: u8) -> Vec<Level> {
    let mut l: Vec<Level> = Vec::new();

    for i in (0..8).rev() {
        if (v & (1 << i)) > 0 {
            l.push(Level::High);
        } else {
            l.push(Level::Low);
        }
    }

    l
}
