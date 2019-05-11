use core::cell::RefCell;

use f3::hal::prelude::*;

use clerk::{Direction, DisplayHardwareLayer, Level};

pub struct ExternPin<T>(RefCell<T>);

impl<T> ExternPin<T> {
    pub fn new(p: T) -> Self {
        ExternPin(RefCell::new(p))
    }
}

impl<P> DisplayHardwareLayer for ExternPin<P>
where
    P: _embedded_hal_digital_OutputPin
{
    fn init(&self) {
        // self.0.export().unwrap();
        // self.0.into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);
    }

    fn cleanup(&self) {
        // self.0.unexport().unwrap();
    }

    fn set_direction(&self, _direction: Direction) {
        // let native_direction = match direction {
        //     Direction::In => sysfs_gpio::Direction::In,
        //     Direction::Out => sysfs_gpio::Direction::Out,
        // };

        // self.0.set_direction(native_direction).unwrap();
    }

    fn set_level(&self, level: Level) {
        let mut mut_pin = self.0.borrow_mut();
        match level {
            Level::High => mut_pin.set_high(),
            Level::Low => mut_pin.set_low(),
        };
    }

    fn get_value(&self) -> u8 {
        // self.0.get_value().unwrap()
        0
    }
}
