extern crate clerk;

use std::cell::RefCell;
use std::collections::VecDeque;

use clerk::{DefaultLines, Delay, Display, DisplayControlBuilder, EntryModeBuilder,
            FunctionSetBuilder, Init, ReadMode, Receive, SeekCgRamFrom, SeekFrom, Send, ShiftTo,
            WriteMode};

struct ConnectionMock {
    init_calls: RefCell<u8>,
    send_bytes: RefCell<Vec<WriteMode>>,
    receivable_bytes: RefCell<VecDeque<u8>>,
}

impl Default for ConnectionMock {
    fn default() -> Self {
        ConnectionMock {
            init_calls: RefCell::new(0),
            send_bytes: RefCell::new(vec![]),
            receivable_bytes: RefCell::new(VecDeque::new()),
        }
    }
}

impl ConnectionMock {
    fn set_read_value(&self, value: u8) {
        self.receivable_bytes.borrow_mut().push_back(value)
    }
}

impl Init for ConnectionMock {
    fn init(&self) {
        let mut init_calls = self.init_calls.borrow_mut();

        *init_calls += 1;
    }
}

impl Send for ConnectionMock {
    fn send(&self, mode: WriteMode) {
        let mut send_bytes = self.send_bytes.borrow_mut();

        send_bytes.push(mode);
    }
}

impl Receive for ConnectionMock {
    fn receive(&self, _: ReadMode) -> u8 {
        self.receivable_bytes.borrow_mut().pop_front().unwrap()
    }
}

pub struct CustomDelayMock;

impl Delay for CustomDelayMock {
    fn delay_ns(_: u16) {
        // mhh
    }
}

fn setup_display() -> Display<ConnectionMock, DefaultLines> {
    Display::new(ConnectionMock::default())
}

#[test]
fn init() {
    let lcd = setup_display();

    lcd.init(&FunctionSetBuilder::default());

    let connection = lcd.get_connection();

    let init_calls = connection.init_calls.borrow_mut();
    assert_eq!(*init_calls, 1);

    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0x33));
    assert_eq!(send_bytes[1], WriteMode::Command(0x32));
    assert_eq!(send_bytes[2], WriteMode::Command(0x20));
    assert_eq!(send_bytes[3], WriteMode::Command(0x01));
}

#[test]
fn set_entry_mode() {
    let lcd = setup_display();

    lcd.set_entry_mode(&EntryModeBuilder::default());

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b0000_0110));
}

#[test]
fn test_set_display_control() {
    let lcd = setup_display();

    lcd.set_display_control(&DisplayControlBuilder::default());

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b0000_1100));
}

#[test]
fn test_shift_cursor_left() {
    let mut lcd = setup_display();

    lcd.shift_cursor(ShiftTo::Left(1));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b0001_0000));
}

#[test]
fn test_shift_cursor_left_with_zero_offset() {
    let mut lcd = setup_display();

    lcd.shift_cursor(ShiftTo::Left(0));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes.len(), 0);
}

#[test]
fn test_shift_cursor_right() {
    let mut lcd = setup_display();

    lcd.shift_cursor(ShiftTo::Right(1));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b0001_0100));
}

#[test]
fn test_shift_cursor_right_multiple() {
    let mut lcd = setup_display();

    lcd.shift_cursor(ShiftTo::Right(2));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b0001_0100));
    assert_eq!(send_bytes[1], WriteMode::Command(0b0001_0100));
}

#[test]
fn test_shift_cursor_right_with_zero_offset() {
    let mut lcd = setup_display();

    lcd.shift_cursor(ShiftTo::Right(0));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes.len(), 0);
}

#[test]
fn test_shift_left() {
    let lcd = setup_display();

    lcd.shift(ShiftTo::Left(1));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b0001_1000));
}

#[test]
fn test_shift_right() {
    let lcd = setup_display();

    lcd.shift(ShiftTo::Right(1));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b0001_1100));
}

#[test]
fn test_clear() {
    let lcd = setup_display();

    lcd.clear();

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0x01));
}

#[test]
fn test_seek_from_home() {
    let mut lcd = setup_display();

    lcd.seek(SeekFrom::Home(3));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b1000_0011));
}

#[test]
fn test_seek_from_current() {
    let mut lcd = setup_display();

    lcd.seek(SeekFrom::Home(2));
    lcd.seek(SeekFrom::Current(1));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b1000_0010));
    assert_eq!(send_bytes[1], WriteMode::Command(0b1000_0011));
}

#[test]
fn test_seek_from_line() {
    let mut lcd = setup_display();

    lcd.seek(SeekFrom::Line {
        line: DefaultLines::Two,
        offset: 3,
    });

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b1100_0011));
}

#[test]
fn test_set_cgram_address_from_home() {
    let lcd = setup_display();

    let lcd = lcd.set_cgram_address(3);

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b0100_0011));
}

#[test]
fn test_seek_cgram_from_current() {
    let lcd = setup_display();

    let mut lcd = lcd.set_cgram_address(2);
    lcd.seek(SeekCgRamFrom::Current(1));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b0100_0010));
    assert_eq!(send_bytes[1], WriteMode::Command(0b0100_0011));
}

#[test]
fn test_write() {
    let mut lcd = setup_display();

    lcd.write(123);

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Data(123));
}

#[test]
fn test_write_updates_address_counter() {
    let mut lcd = setup_display();

    lcd.seek(SeekFrom::Home(0));
    lcd.write(12);
    lcd.write(34);
    lcd.seek(SeekFrom::Current(0));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[3], WriteMode::Command(0b1000_0010));
}

#[test]
fn test_write_message() {
    let mut lcd = setup_display();

    lcd.write_message("Hi");

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Data(b'H'));
    assert_eq!(send_bytes[1], WriteMode::Data(b'i'));
}

#[test]
fn test_write_message_increments_address_counter() {
    let mut lcd = setup_display();

    lcd.write_message("Hi");
    lcd.seek(SeekFrom::Current(0));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[2], WriteMode::Command(0b1000_0010));
}

#[test]
fn test_read() {
    let expected = 42;

    let connection = ConnectionMock::default();
    connection.set_read_value(expected);

    let mut lcd: Display<ConnectionMock, DefaultLines> = Display::new(connection);
    let input = lcd.read_byte();
    assert_eq!(input, expected);
}

#[test]
fn test_read_increments_address_counter() {
    let connection = ConnectionMock::default();

    connection.set_read_value(4);
    connection.set_read_value(2);

    let mut lcd: Display<ConnectionMock, DefaultLines> = Display::new(connection);

    lcd.read_byte();
    lcd.seek(SeekFrom::Current(0));

    lcd.read_byte();
    lcd.seek(SeekFrom::Current(0));

    let connection = lcd.get_connection();
    let send_bytes = connection.send_bytes.borrow_mut();
    assert_eq!(send_bytes[0], WriteMode::Command(0b1000_0001));
    assert_eq!(send_bytes[1], WriteMode::Command(0b1000_0010));
}
