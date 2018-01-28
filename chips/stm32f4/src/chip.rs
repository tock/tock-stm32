//! Interrupt mapping and DMA channel setup.

use cortexm4;
use exti;
use kernel::Chip;

pub struct Stm32f4 {
    pub mpu: cortexm4::mpu::MPU,
    pub systick: &'static cortexm4::systick::SysTick,
}

impl Stm32f4 {
    pub unsafe fn new() -> Stm32f4 {
        Stm32f4 {
            mpu: cortexm4::mpu::MPU::new(),
            systick: cortexm4::systick::SysTick::new(),
        }
    }
}

impl Chip for Stm32f4 {
    type MPU = cortexm4::mpu::MPU;
    type SysTick = cortexm4::systick::SysTick;

    fn service_pending_interrupts(&mut self) {
        use nvic::*;

        unsafe {
            loop {
                if let Some(interrupt) = cortexm4::nvic::next_pending() {
                    match interrupt {
                        EXTI0 => exti::EXTI[0].handle_interrupt(),
                        EXTI1 => exti::EXTI[1].handle_interrupt(),
                        EXTI2 => exti::EXTI[2].handle_interrupt(),
                        EXTI3 => exti::EXTI[3].handle_interrupt(),
                        EXTI4 => exti::EXTI[4].handle_interrupt(),
                        _ => {
                            panic!("unhandled interrupt {}", interrupt);
                        }
                    }
                    let n = cortexm4::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm4::nvic::has_pending() }
    }

    fn mpu(&self) -> &cortexm4::mpu::MPU {
        &self.mpu
    }

    fn systick(&self) -> &cortexm4::systick::SysTick {
        self.systick
    }

    fn prepare_for_sleep(&self) {
        /*if pm::deep_sleep_ready() {
            unsafe {
                cortexm4::scb::set_sleepdeep();
            }
        } else {
            unsafe {
                cortexm4::scb::unset_sleepdeep();
            }
        }*/
    }
}
