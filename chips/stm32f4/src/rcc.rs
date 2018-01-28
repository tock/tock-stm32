//! Reset and clock control

use bitfield::Bit;
use kernel::common::VolatileCell;

#[repr(C)]
struct Registers {
    cr: VolatileCell<u32>,
    pllcfgr: VolatileCell<u32>,
    cfgr: VolatileCell<u32>,
    cir: VolatileCell<u32>,
    ahb1rstr: VolatileCell<u32>,
    ahb2rstr: VolatileCell<u32>,
    ahb3rstr: VolatileCell<u32>,
    _reserved0: u32,
    apb1rstr: VolatileCell<u32>,
    apb2rstr: VolatileCell<u32>,
    _reserved1: [u32; 2],
    ahb1enr: VolatileCell<u32>,
    ahb2enr: VolatileCell<u32>,
    ahb3enr: VolatileCell<u32>,
    _reserved2: u32,
    apb1enr: VolatileCell<u32>,
    apb2enr: VolatileCell<u32>,
    _reserved3: [u32; 2],
    ahb1lpenr: VolatileCell<u32>,
    ahb2lpenr: VolatileCell<u32>,
    ahb3lpenr: VolatileCell<u32>,
    _reserved4: u32,
    apb1lpenr: VolatileCell<u32>,
    apb2lpenr: VolatileCell<u32>,
    _reserved5: [u32; 2],
    bdcr: VolatileCell<u32>,
    csr: VolatileCell<u32>,
    _reserved6: [u32; 2],
    sscgr: VolatileCell<u32>,
    plli2scfgr: VolatileCell<u32>,
}

const RCC_BASE_ADDRESS: usize = 0x4002_3800;

fn rcc() -> &'static Registers {
    unsafe { &*(RCC_BASE_ADDRESS as *const Registers) }
}

#[derive(Copy, Clone, Debug)]
pub enum Clock {
    AHB1(AHB1Clock),
    AHB2,
    AHB3,
    APB1,
    APB2,
    AHB1LP,
    AHB2LP,
    AHB3LP,
}

#[derive(Copy, Clone, Debug)]
pub enum AHB1Clock {
    GPIOA,
    GPIOB,
    GPIOC,
    GPIOD,
}

pub fn enable_clock(clock: Clock, enable: bool) {
    let rcc = rcc();

    let (reg, c) = match clock {
        Clock::AHB1(c) => (&rcc.ahb1enr, c),
        _ => return,
    };

    let mut val = reg.get();
    val.set_bit(c as usize, enable);
    reg.set(val);
}
