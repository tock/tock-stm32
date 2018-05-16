use cortexm3;
use cortexm3::nvic;
use kernel;
use stm32;
use stm32::nvic::*;

pub struct STM32F1 {
    mpu: (),
    systick: cortexm3::systick::SysTick,
}

impl STM32F1 {
    pub unsafe fn new() -> STM32F1 {
        STM32F1 {
            mpu: (),
            systick: cortexm3::systick::SysTick::new(),
        }
    }
}

impl kernel::Chip for STM32F1 {
    type MPU = ();
    type SysTick = cortexm3::systick::SysTick;

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn systick(&self) -> &Self::SysTick {
        &self.systick
    }

    fn service_pending_interrupts(&mut self) {
        unsafe {
            while let Some(interrupt) = nvic::next_pending() {
                match interrupt {
                    TIM2 => stm32::timer::TIMER2.handle_interrupt(),
                    USART1 => stm32::usart::USART1.handle_interrupt(),
                    _ => panic!("unhandled interrupt {}", interrupt),
                }
                let n = nvic::Nvic::new(interrupt);
                n.clear_pending();
                n.enable();
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { nvic::has_pending() }
    }

    fn sleep(&self) {}
}
