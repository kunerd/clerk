use std::{thread, time};

use clerk::Delay;

pub struct CustomDelay;

impl Delay for CustomDelay {
    fn delay_ns(ns: u16) {
        thread::sleep(time::Duration::new(0, ns as u32));
    }
}
