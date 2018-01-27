//! Basic implementation of ARM semihosting
//!
//! http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0471g/Bgbjjgij.html

use self::SysOp::*;
use core::cell::Cell;
use core::fmt::{self, Write};
use kernel::hil::uart;

enum SysOp {
    SysOpen = 0x01,
    SysClose,
    SysWrite = 0x05,
}

fn syscall(cmd: SysOp, arg: *const ()) -> i32 {
    let ret_val: i32;

    unsafe {
        // TODO Investigate this
        // https://github.com/rust-lang/rust/commit/e32da1275300a437490dd2e37ec49d7f11e3d780
        asm!("bkpt 0xab" : "={r0}"(ret_val)  : "{r1}"(arg), "{r0}"(cmd as u32) : "r0" : "volatile");
    }

    ret_val
}

pub enum OpenMode {
    Read,
    ReadBinary,
    ReadUpdate,
    ReadUpdateBinary,
    Write,
    WriteBinary,
    WriteUpdate,
    WriteUpdateBinary,
    Append,
    AppendBinary,
    AppendUpdate,
    AppendUpdateBinary,
}

pub struct Channel {
    fd: Cell<i32>,
    client: Cell<Option<&'static uart::Client>>,
}

impl Channel {
    pub const fn new() -> Channel {
        Channel {
            fd: Cell::new(-1),
            client: Cell::new(None),
        }
    }

    pub fn close(&self) {
        let fd = self.fd.get();

        if fd >= 0 {
            let arg: i32 = fd;
            syscall(SysClose, &arg as *const i32 as *const ());
            self.fd.set(-1);
        }
    }

    pub fn open(&self, name: &str, mode: OpenMode) -> i32 {
        self.close();

        let arg: (u32, u32, u32) = (name.as_ptr() as u32, mode as u32, name.len() as u32);
        let fd = syscall(SysOpen, &arg as *const (u32, u32, u32) as *const ());
        self.fd.set(fd);
        fd
    }
}

impl Write for Channel {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let fd = self.fd.get();

        if fd < 0 {
            Err(fmt::Error)
        } else {
            let arg: (i32, u32, u32) = (fd, s.as_ptr() as usize as u32, s.len() as u32);
            syscall(SysWrite, &arg as *const (i32, u32, u32) as *const ());
            Ok(())
        }
    }
}

impl uart::UART for Channel {
    fn set_client(&self, client: &'static uart::Client) {
        self.client.set(Some(client))
    }

    fn init(&self, _: uart::UARTParams) {}

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        if tx_len == 0 {
            return;
        }

        let fd = self.fd.get();

        if fd >= 0 {
            let arg: (i32, u32, u32) = (fd, tx_data.as_ptr() as u32, tx_len as u32);
            syscall(SysWrite, &arg as *const (i32, u32, u32) as *const ());
        }

        // Signal client write done
        self.client.get().map(move |client| {
            client.transmit_complete(tx_data, uart::Error::CommandComplete);
        });
    }

    fn receive(&self, _: &'static mut [u8], _: usize) {}
}
