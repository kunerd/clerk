use std::{thread, time};

use clerk::Delay;
use clerk::DelayUs;

pub struct CustomDelay;

impl Delay for CustomDelay {}

impl DelayUs<u8> for CustomDelay {
    fn delay_us(&mut self, us: u8) {
        thread::sleep(time::Duration::new(us.into(), 0));
    }
}
