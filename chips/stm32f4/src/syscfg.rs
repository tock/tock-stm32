//! System configuration controller

use kernel::common::VolatileCell;

#[repr(C)]
pub struct Registers {
    _memrmp: VolatileCell<u32>,
    _pmc: VolatileCell<u32>,
    pub exticr1: VolatileCell<u32>,
    pub exticr2: VolatileCell<u32>,
    pub exticr3: VolatileCell<u32>,
    pub exticr4: VolatileCell<u32>,
    _cmpcr: VolatileCell<u32>,
}

const BASE_ADDRESS: usize = 0x4001_3800;

pub fn syscfg() -> &'static Registers {
    unsafe { &*(BASE_ADDRESS as *const Registers) }
}
