use sysfs_gpio;

use clerk::{Direction, DisplayHardwareLayer, Level};

pub struct ExternPin(sysfs_gpio::Pin);

impl ExternPin {
    pub fn new(i: u64) -> Self {
        ExternPin(sysfs_gpio::Pin::new(i))
    }
}

impl DisplayHardwareLayer for ExternPin {
    fn init(&self) {
        self.0.export().unwrap();
        self.0.set_direction(sysfs_gpio::Direction::Out).unwrap();
    }

    fn cleanup(&self) {
        self.0.unexport().unwrap();
    }

    fn set_direction(&self, direction: Direction) {
        let native_direction = match direction {
            Direction::In => sysfs_gpio::Direction::In,
            Direction::Out => sysfs_gpio::Direction::Out,
        };

        self.0.set_direction(native_direction).unwrap();
    }

    fn set_level(&self, level: Level) {
        let value = match level {
            Level::High => 1,
            Level::Low => 0,
        };

        self.0.set_value(value).unwrap();
    }

    fn get_value(&self) -> u8 {
        self.0.get_value().unwrap()
    }
}
