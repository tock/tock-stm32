//! Various helper methods, traits

use bitfield;
use kernel::common::VolatileCell;

pub trait BitRange<T> {
    fn bit_range(&self, msb: usize, lsb: usize) -> T;
    fn set_bit_range(&self, msb: usize, lsb: usize, value: T);
}

pub trait Bit {
    fn bit(&self, bit: usize) -> bool;
    fn set_bit(&self, bit: usize, value: bool);
}

impl BitRange<u32> for VolatileCell<u32> {
    fn bit_range(&self, msb: usize, lsb: usize) -> u32 {
        bitfield::BitRange::bit_range(&(self.get()), msb, lsb)
    }

    fn set_bit_range(&self, msb: usize, lsb: usize, value: u32) {
        let mut val = self.get();
        bitfield::BitRange::set_bit_range(&mut val, msb, lsb, value);
        self.set(val);
    }
}

impl Bit for VolatileCell<u32> {
    fn bit(&self, bit: usize) -> bool {
        bitfield::Bit::bit(&(self.get()), bit)
    }

    fn set_bit(&self, bit: usize, value: bool) {
        let mut val = self.get();
        bitfield::Bit::set_bit(&mut val, bit, value);
        self.set(val);
    }
}
