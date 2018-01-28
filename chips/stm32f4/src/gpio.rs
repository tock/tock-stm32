//! GPIO controller

use core::cell::Cell;
use exti;
use helpers::{Bit, BitRange};
use kernel::common::VolatileCell;
use kernel::common::static_ref::StaticRef;
use kernel::hil;
use kernel::hil::gpio::{InterruptMode, Pin};

const BASE_ADDRESS: usize = 0x4002_0000;
const SIZE: usize = 0x400;

pub static mut PA0: GPIOPin = GPIOPin::new(PortNum::A, 0);
pub static mut PD12: GPIOPin = GPIOPin::new(PortNum::D, 12);
pub static mut PD13: GPIOPin = GPIOPin::new(PortNum::D, 13);
pub static mut PD14: GPIOPin = GPIOPin::new(PortNum::D, 14);
pub static mut PD15: GPIOPin = GPIOPin::new(PortNum::D, 15);

#[cfg_attr(rustfmt, rustfmt_skip)]
pub static mut PORT_MAP: PortMap = unsafe {
    PortMap {
        pins: [
            Some(&PA0), None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        ],
    }
};

#[derive(Copy, Clone)]
pub enum PortNum {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
}

#[derive(Copy, Clone)]
pub enum Mode {
    Input,
    Output,
    AlternateFunction,
    Analog,
}

#[derive(Copy, Clone)]
pub enum OutputMode {
    PushPull,
    OpenDrain,
}

#[derive(Copy, Clone)]
pub enum OutputSpeed {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Copy, Clone)]
pub enum PullUpDownMode {
    PullNone,
    PullUp,
    PullDown,
}

#[derive(Copy, Clone)]
pub enum AlternateFunction {
    AF0,
    AF1,
    AF2,
    AF3,
    AF4,
    AF5,
    AF6,
    AF7,
    AF8,
    AF9,
    AF10,
    AF11,
    AF12,
    AF13,
    AF14,
    AF15,
}

#[repr(C)]
struct Registers {
    moder: VolatileCell<u32>,
    otyper: VolatileCell<u32>,
    ospeedr: VolatileCell<u32>,
    pupdr: VolatileCell<u32>,
    idr: VolatileCell<u32>,
    odr: VolatileCell<u32>,
    bsrr: VolatileCell<u32>,
    lckr: VolatileCell<u32>,
    afrl: VolatileCell<u32>,
    afrh: VolatileCell<u32>,
}

pub struct GPIOPin {
    regs: StaticRef<Registers>,
    pub pin: usize,
    pub port: PortNum,
    client_data: Cell<usize>,
    client: Cell<Option<&'static hil::gpio::Client>>,
}

impl GPIOPin {
    const fn new(port: PortNum, pin: usize) -> GPIOPin {
        GPIOPin {
            regs: unsafe {
                StaticRef::new((BASE_ADDRESS + ((port as usize) * SIZE)) as *const Registers)
            },
            pin: pin,
            port: port,
            client_data: Cell::new(0),
            client: Cell::new(None),
        }
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    pub fn set_mode(&self, mode: Mode) {
        self.regs
            .moder
            .set_bit_range(self.pin * 2 + 1, self.pin * 2, mode as u32);
    }

    pub fn set_output_type(&self, mode: OutputMode) {
        self.regs
            .otyper
            .set_bit_range(self.pin, self.pin, mode as u32);
    }

    pub fn set_output_speed(&self, speed: OutputSpeed) {
        self.regs
            .ospeedr
            .set_bit_range(self.pin * 2 + 1, self.pin * 2, speed as u32);
    }

    pub fn set_pull_mode(&self, pull_mode: PullUpDownMode) {
        self.regs
            .pupdr
            .set_bit_range(self.pin * 2 + 1, self.pin * 2, pull_mode as u32);
    }

    pub fn set_alternate_function(&self, function: AlternateFunction) {
        let (afr_register, pos): (&VolatileCell<u32>, usize) = match self.pin {
            0...7 => (&self.regs.afrl, self.pin * 4),
            _ => (&self.regs.afrh, self.pin * 4 - 32),
        };

        afr_register.set_bit_range(pos + 3, pos, function as u32);
    }

    pub fn set_data(&self, high: bool) {
        self.regs.odr.set_bit(self.pin, high);
    }

    pub fn get_data(&self) -> bool {
        self.regs.idr.bit(self.pin)
    }

    pub fn handle_interrupt(&self) {
        self.client.get().map(|client| {
            client.fired(self.client_data.get());
        });
    }
}

impl Pin for GPIOPin {
    fn make_output(&self) {
        self.set_mode(Mode::Output);
    }

    fn make_input(&self) {
        self.set_mode(Mode::Input);
    }

    fn disable(&self) {
        self.set_mode(Mode::Input);
    }

    fn set(&self) {
        self.set_data(true);
    }

    fn clear(&self) {
        self.set_data(false);
    }

    fn toggle(&self) {
        self.set_data(!self.get_data());
    }

    fn read(&self) -> bool {
        self.get_data()
    }

    fn enable_interrupt(&self, identifier: usize, mode: InterruptMode) {
        let (raising_edge, falling_edge) = match mode {
            InterruptMode::RisingEdge => (true, false),
            InterruptMode::FallingEdge => (false, true),
            InterruptMode::EitherEdge => (true, true),
        };

        self.set_mode(Mode::Input);
        self.client_data.set(identifier);

        unsafe {
            exti::EXTI[self.pin].set_client(PORT_MAP.get_pin(self.port, self.pin));
            exti::EXTI[self.pin].set_trigger(raising_edge, falling_edge);
            exti::EXTI[self.pin].enable_interrupt(true);
        }
    }

    fn disable_interrupt(&self) {
        unsafe {
            exti::EXTI[self.pin].enable_interrupt(false);
            exti::EXTI[self.pin].set_trigger(false, false);
            exti::EXTI[self.pin].set_client(None);
        }
    }
}

impl hil::gpio::PinCtl for GPIOPin {
    fn set_input_mode(&self, input_mode: hil::gpio::InputMode) {
        let pull_mode: PullUpDownMode = match input_mode {
            hil::gpio::InputMode::PullUp => PullUpDownMode::PullUp,
            hil::gpio::InputMode::PullDown => PullUpDownMode::PullDown,
            hil::gpio::InputMode::PullNone => PullUpDownMode::PullNone,
        };

        self.set_mode(Mode::Input);
        self.set_pull_mode(pull_mode);
    }
}

pub struct PortMap<'a> {
    pins: [Option<&'a GPIOPin>; 160],
}

impl<'a> PortMap<'a> {
    pub fn get_pin(&self, port: PortNum, pin: usize) -> Option<&'a GPIOPin> {
        self.pins[((port as usize) * 16 + pin)]
    }
}
