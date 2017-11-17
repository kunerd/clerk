use core::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Address(u8);

impl From<u8> for Address {
    fn from(val: u8) -> Self {
        Address(val % 128)
    }
}

impl From<Address> for u8 {
    fn from(addr: Address) -> Self {
        addr.0
    }
}

impl Default for Address {
    fn default() -> Self {
        Address(0)
    }
}

impl Add for Address {
    type Output = Address;

    fn add(self, other: Address) -> Address {
        let val = (self.0 + other.0) % 128;
        Address(val)
    }
}

impl AddAssign for Address {
    fn add_assign(&mut self, other: Address) {
        *self = *self + other;
    }
}

impl Sub for Address {
    type Output = Address;

    fn sub(self, other: Address) -> Address {
        let val = if self.0 >= other.0 {
            self.0 - other.0
        } else {
            128 - (other.0 - self.0)
        };

        Address(val)
    }
}

impl SubAssign for Address {
    fn sub_assign(&mut self, other: Address) {
        *self = *self - other;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_with_overflow() {
        let a = Address::from(128);
        assert_eq!(a, Address(0));

        let a = Address::from(127 + 3);
        assert_eq!(a, Address(2));
    }

    #[test]
    fn add_without_overflow() {
        let a = Address(5);
        let b = Address(10);

        assert_eq!(a + b, Address(15));
    }

    #[test]
    fn add_with_overflow() {
        let a = Address(120);
        let b = Address(10);

        assert_eq!(a + b, Address(2));
    }

    #[test]
    fn sub_without_overflow() {
        let a = Address(20);
        let b = Address(10);

        assert_eq!(a - b, Address(10));
    }

    #[test]
    fn sub_with_overflow() {
        let a = Address(10);
        let b = Address(20);

        assert_eq!(a - b, Address(118));
    }

    #[test]
    fn add_assign_without_overflow() {
        let mut a = Address(10);
        let b = Address(20);

        a += b;

        assert_eq!(a, Address(30));
    }

    #[test]
    fn add_assign_with_overflow() {
        let mut a = Address(120);
        let b = Address(8);

        a += b;

        assert_eq!(a, Address(0));
    }

    #[test]
    fn sub_assign_without_overflow() {
        let mut a = Address(123);
        let b = Address(23);

        a -= b;

        assert_eq!(a, Address(100));
    }

    #[test]
    fn sub_assign_with_overflow() {
        let mut a = Address(5);
        let b = Address(7);

        a -= b;

        assert_eq!(a, Address(126));
    }
}
