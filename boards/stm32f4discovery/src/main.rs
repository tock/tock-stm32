#![no_std]
#![no_main]
#![feature(asm, const_fn, lang_items, const_cell_new, core_intrinsics)]

extern crate capsules;
#[allow(unused_imports)]
#[macro_use(debug, debug_gpio, static_init)]
extern crate kernel;
extern crate stm32f4;

#[macro_use]
pub mod io;
pub mod semihosting;

use stm32f4::rcc;

// State for loading apps.

const NUM_PROCS: usize = 4;

// how should the kernel respond when a process faults
const FAULT_RESPONSE: kernel::process::FaultResponse = kernel::process::FaultResponse::Panic;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 16384] = [0; 16384];

static mut PROCESSES: [Option<&mut kernel::Process<'static>>; NUM_PROCS] = [None, None, None, None];

struct Discovery {
    console: &'static capsules::console::Console<'static, semihosting::Channel>,
    gpio: &'static capsules::gpio::GPIO<'static, stm32f4::gpio::GPIOPin>,
    led: &'static capsules::led::LED<'static, stm32f4::gpio::GPIOPin>,
    button: &'static capsules::button::Button<'static, stm32f4::gpio::GPIOPin>,
    ipc: kernel::ipc::IPC,
}

impl kernel::Platform for Discovery {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

#[no_mangle]
pub unsafe fn reset_handler() {
    stm32f4::init();

    let console = static_init!(
        capsules::console::Console<semihosting::Channel>,
        capsules::console::Console::new(
            &io::STD_OUT,
            115200,
            &mut capsules::console::WRITE_BUF,
            &mut capsules::console::READ_BUF,
            kernel::Grant::create()
        )
    );
    kernel::hil::uart::UART::set_client(&io::STD_OUT, console);

    io::std_out_init();
    console.initialize();

    // Attach the kernel debug interface to this console
    let kc = static_init!(capsules::console::App, capsules::console::App::default());
    kernel::debug::assign_console_driver(Some(console), kc);

    // GPIOs
    rcc::enable_clock(rcc::Clock::AHB1(rcc::AHB1Clock::GPIOA), true);
    rcc::enable_clock(rcc::Clock::AHB1(rcc::AHB1Clock::GPIOD), true);

    let gpio_pins = static_init!(
        [&'static stm32f4::gpio::GPIOPin; 5],
        [
            &stm32f4::gpio::PA0,
            &stm32f4::gpio::PD12,
            &stm32f4::gpio::PD13,
            &stm32f4::gpio::PD14,
            &stm32f4::gpio::PD15,
        ]
    );

    let gpio = static_init!(
        capsules::gpio::GPIO<'static, stm32f4::gpio::GPIOPin>,
        capsules::gpio::GPIO::new(gpio_pins)
    );

    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    // LEDs
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let led_pins = static_init!(
        [(&'static stm32f4::gpio::GPIOPin,capsules::led::ActivationMode); 1],
        [(&stm32f4::gpio::PD13,capsules::led::ActivationMode::ActiveHigh)]
    );

    let led = static_init!(
        capsules::led::LED<'static, stm32f4::gpio::GPIOPin>,
        capsules::led::LED::new(led_pins)
    );

    // Buttons
    let button_pins = static_init!(
        [(&'static stm32f4::gpio::GPIOPin, capsules::button::GpioMode); 1],
        [(
            &stm32f4::gpio::PA0,
            capsules::button::GpioMode::LowWhenPressed
        )]
    );

    let button = static_init!(
        capsules::button::Button<'static, stm32f4::gpio::GPIOPin>,
        capsules::button::Button::new(button_pins, kernel::Grant::create())
    );

    for &(btn, _) in button_pins.iter() {
        btn.set_client(button);
    }

    let discovery = Discovery {
        console: console,
        gpio: gpio,
        led: led,
        button: button,
        ipc: kernel::ipc::IPC::new(),
    };

    let mut chip = stm32f4::chip::Stm32f4::new();

    debug!("Initialization complete. Entering main loop");
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
    }
    kernel::process::load_processes(
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
    );
    kernel::main(&discovery, &mut chip, &mut PROCESSES, &discovery.ipc);
}
