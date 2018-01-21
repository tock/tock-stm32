use core::cell::Cell;
use core::ops::{Index, IndexMut};
use gpio;
use helpers::{Bit, BitRange};
use kernel::common::VolatileCell;
use syscfg::syscfg;

const BASE_ADDRESS: usize = 0x04001_3C00;

fn exti() -> &'static Registers {
    unsafe { &*(BASE_ADDRESS as *const Registers) }
}

pub static mut EXTI: Exti = Exti {
    lines: [
        ExtiLine::new(0),
        ExtiLine::new(1),
        ExtiLine::new(2),
        ExtiLine::new(3),
        ExtiLine::new(4),
    ],
};

pub struct Exti {
    pub lines: [ExtiLine; 5],
}

impl Index<usize> for Exti {
    type Output = ExtiLine;

    fn index(&self, index: usize) -> &ExtiLine {
        &self.lines[index]
    }
}

impl IndexMut<usize> for Exti {
    fn index_mut(&mut self, index: usize) -> &mut ExtiLine {
        &mut self.lines[index]
    }
}

#[repr(C)]
struct Registers {
    imr: VolatileCell<u32>,
    emr: VolatileCell<u32>,
    rtsr: VolatileCell<u32>,
    ftsr: VolatileCell<u32>,
    swier: VolatileCell<u32>,
    pr: VolatileCell<u32>,
}

pub struct ExtiLine {
    line: usize,
    gpio: Cell<Option<&'static gpio::GPIOPin>>,
}

impl ExtiLine {
    const fn new(line: usize) -> ExtiLine {
        ExtiLine {
            line: line,
            gpio: Cell::new(None),
        }
    }

    pub fn enable_interrupt(&self, enable: bool) {
        exti().imr.set_bit(self.line, enable);
    }

    pub fn set_trigger(&self, raising: bool, falling: bool) {
        exti().rtsr.set_bit(self.line, raising);
        exti().ftsr.set_bit(self.line, falling);
    }

    pub fn has_pending(&self) -> bool {
        exti().pr.bit(self.line)
    }

    fn set_exti_source(&self, port: gpio::PortNum) {
        let port = port as u32;
        let pos = ((port % 3) * 4) as usize;
        let exticr = match self.line {
            0...3 => &syscfg().exticr1,
            4...7 => &syscfg().exticr2,
            8...11 => &syscfg().exticr3,
            12...15 => &syscfg().exticr4,
            _ => return (),
        };
        exticr.set_bit_range(pos + 3, pos, port);
    }

    pub fn set_client(&self, gpio: Option<&'static gpio::GPIOPin>) {
        match gpio {
            Some(gpio) if (gpio.pin == self.line) => {
                self.gpio.set(Some(gpio));
                self.set_exti_source(gpio.port);
            }
            _ => (),
        }
    }

    pub fn handle_interrupt(&self) {
        exti().pr.set(1 << self.line); // Clear pending bit by writing 1

        self.gpio.get().map(|gpio| {
            gpio.handle_interrupt();
        });
    }
}
