use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(PartialEq, PartialOrd, Debug)]
pub struct Address<T>(u8, PhantomData<T>);

pub trait Overflow {
    const UPPER_BOUND: u8;
}

impl<T: Overflow> From<u8> for Address<T> {
    fn from(val: u8) -> Self {
        Address(val % T::UPPER_BOUND, PhantomData)
    }
}

impl<T> From<Address<T>> for u8 {
    fn from(addr: Address<T>) -> Self {
        addr.0
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(expl_impl_clone_on_copy))]
impl<T> Clone for Address<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Address<T> {}

impl<T: Overflow> Default for Address<T> {
    fn default() -> Self {
        Address::from(0)
    }
}

impl<T: Overflow> Add for Address<T> {
    type Output = Address<T>;

    fn add(self, other: Address<T>) -> Self::Output {
        let val = self.0 + other.0;
        Address::from(val)
    }
}

impl<T: Overflow> AddAssign for Address<T> {
    fn add_assign(&mut self, other: Address<T>) {
        *self = *self + other;
    }
}

impl<T: Overflow> Sub for Address<T> {
    type Output = Address<T>;

    fn sub(self, other: Address<T>) -> Self::Output {
        let val = if self.0 >= other.0 {
            self.0 - other.0
        } else {
            T::UPPER_BOUND - (other.0 - self.0)
        };

        Address::from(val)
    }
}

impl<T: Overflow> SubAssign for Address<T> {
    fn sub_assign(&mut self, other: Address<T>) {
        *self = *self - other;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, PartialOrd)]
    struct DdRamMock;
    impl Overflow for DdRamMock {
        const UPPER_BOUND: u8 = 128;
    }

    #[test]
    fn from_with_overflow() {
        let a: Address<DdRamMock> = Address::from(128);
        assert_eq!(a, Address::from(0));

        let a: Address<DdRamMock> = Address::from(127 + 3);
        assert_eq!(a, Address::from(2));
    }

    #[test]
    fn add_without_overflow() {
        let a: Address<DdRamMock> = Address::from(5);
        let b = Address::from(10);

        assert_eq!(a + b, Address::from(15));
    }

    #[test]
    fn add_with_overflow() {
        let a: Address<DdRamMock> = Address::from(120);
        let b = Address::from(10);

        assert_eq!(a + b, Address::from(2));
    }

    #[test]
    fn sub_without_overflow() {
        let a: Address<DdRamMock> = Address::from(20);
        let b = Address::from(10);

        assert_eq!(a - b, Address::from(10));
    }

    #[test]
    fn sub_with_overflow() {
        let a: Address<DdRamMock> = Address::from(10);
        let b = Address::from(20);

        assert_eq!(a - b, Address::from(118));
    }

    #[test]
    fn add_assign_without_overflow() {
        let mut a: Address<DdRamMock> = Address::from(10);
        let b = Address::from(20);

        a += b;

        assert_eq!(a, Address::from(30));
    }

    #[test]
    fn add_assign_with_overflow() {
        let mut a: Address<DdRamMock> = Address::from(120);
        let b = Address::from(8);

        a += b;

        assert_eq!(a, Address::from(0));
    }

    #[test]
    fn sub_assign_without_overflow() {
        let mut a: Address<DdRamMock> = Address::from(123);
        let b = Address::from(23);

        a -= b;

        assert_eq!(a, Address::from(100));
    }

    #[test]
    fn sub_assign_with_overflow() {
        let mut a: Address<DdRamMock> = Address(5, PhantomData);
        let b = Address(7, PhantomData);

        a -= b;

        assert_eq!(a, Address::from(126));
    }
}
