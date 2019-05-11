#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger

use cortex_m::asm;
use cortex_m_rt::entry;

extern crate clerk;

// mod extern_pin;

use f3::hal::delay;
use f3::hal::prelude::*;
use f3::hal::stm32f30x::Peripherals;
use clerk::{CursorBlinking, CursorState, DataPins4Lines, DefaultLines, Delay, Display,
            DisplayControlBuilder, DisplayState, FunctionSetBuilder, LineNumber, Pins, SeekFrom};

// use extern_pin::ExternPin;

pub struct CustomDelay;

impl Delay for CustomDelay {}

#[entry]
fn main() -> ! {
   let p = Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();

    let mut gpiod = p.GPIOD.split(&mut rcc.ahb);
    let pd0 = gpiod
        .pd0
        .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

    let pd1 = gpiod
        .pd1
        .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

    let pd2 = gpiod
        .pd2
        .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

    let pd3 = gpiod
        .pd3
        .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

    let pd4 = gpiod
        .pd4
        .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

    let pd5 = gpiod
        .pd5
        .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

    let pd6 = gpiod
        .pd6
        .into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

    let cp = cortex_m::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let delay = delay::Delay::new(cp.SYST, clocks);

    let pins = Pins {
        register_select: pd0,
        read: pd1,
        enable: pd2,
        data: DataPins4Lines {
            data4: pd3,
            data5: pd4,
            data6: pd5,
            data7: pd6,
        },
    };

    let connection = pins.into_connection::<CustomDelay, delay::Delay>(delay);
    let mut lcd: Display<_, DefaultLines> = Display::new(connection);

    lcd.init(FunctionSetBuilder::default().set_line_number(LineNumber::Two));
    lcd.clear();
    for _ in 0..1000 {
        cortex_m::asm::nop();
    }

    lcd.seek(SeekFrom::Home(0));

    lcd.set_display_control(
        DisplayControlBuilder::default()
            .set_display(DisplayState::On)
            .set_cursor(CursorState::On)
            .set_cursor_blinking(CursorBlinking::On),
    );

    lcd.write_message("Hello");

    lcd.seek(SeekFrom::Line {
        line: DefaultLines::Two,
        offset: 3,
    });

    lcd.write_message("F3 Discovery!");

    lcd.seek(SeekFrom::Home(5));
    for _ in 0..24_000 {
        cortex_m::asm::nop();
    }

    lcd.seek(SeekFrom::Home(0));
    lcd.write_message("Goodbye!");

    loop {
        cortex_m::asm::nop();
    }
}
