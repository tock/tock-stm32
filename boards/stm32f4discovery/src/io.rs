//! Semihosting console for standard and debugging/panic output

use core;
use core::fmt::*;
use kernel::process;
use semihosting;

pub static mut STD_OUT: semihosting::Channel = semihosting::Channel::new();

pub unsafe fn std_out_init() {
    use kernel::common::VolatileCell;
    use kernel::common::static_ref::StaticRef;

    const DHCSR: usize = 0xe000_edf0;
    const C_DEBUGEN_MASK: u32 = 1;

    let dhcsr = StaticRef::new(DHCSR as *const VolatileCell<u32>);

    // Check whether we are controlled by debugger
    if (dhcsr.get() & C_DEBUGEN_MASK) != 0 {
        STD_OUT.open(":tt", semihosting::OpenMode::Write);
    }
}

#[cfg(not(test))]
#[no_mangle]
#[lang = "panic_fmt"]
pub unsafe extern "C" fn panic_fmt(args: Arguments, file: &'static str, line: u32) -> ! {
    let writer = &mut STD_OUT;
    let _ = writer.write_fmt(format_args!(
        "\r\n\nKernel panic at {}:{}:\r\n\t\"",
        file, line
    ));
    let _ = write(writer, args);
    let _ = writer.write_str("\"\r\n");

    // Print version of the kernel
    let _ = writer.write_fmt(format_args!("\tKernel version {}\r\n", "xxx"));

    // Print fault status once
    let procs = &mut process::PROCS;
    if procs.len() > 0 {
        procs[0].as_mut().map(|process| {
            process.fault_str(writer);
        });
    }

    // print data about each process
    let _ = writer.write_fmt(format_args!("\r\n---| App Status |---\r\n"));
    let procs = &mut process::PROCS;
    for idx in 0..procs.len() {
        procs[idx].as_mut().map(|process| {
            process.statistics_str(writer);
        });
    }

    core::intrinsics::breakpoint();
    loop {}
}

#[macro_export]
macro_rules! print {
        ($($arg:tt)*) => (
            {
                use core::fmt::write;
                let writer = unsafe { &mut $crate::io::STD_OUT };
                let _ = write(writer, format_args!($($arg)*));
            }
        );
}

#[macro_export]
macro_rules! println {
        ($fmt:expr) => (print!(concat!($fmt, "\n")));
            ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}
