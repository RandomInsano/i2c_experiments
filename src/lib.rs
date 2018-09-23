//! Experiments in getting Linux's I2C_RDWR ioctlto work in Rust

#![allow(unused)]
#![deny(missing_docs)]

/// Notes:
/// Implementation of i2c ioctls: https://github.com/torvalds/linux/blob/master/drivers/i2c/i2c-dev.c#L439

#[macro_use]
extern crate nix;
extern crate errno;

use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use std::mem;
use std::os::unix::io::AsRawFd;
use std::os::unix::prelude::*;

const I2C_M_RD: u16 = 0x0001; /* read data, from slave to master */
const I2C_M_TEN: u16 = 0x0010; /* this is a ten bit chip address */
const I2C_M_RECV_LEN: u16 = 0x0400; /* length will be first received byte */
const I2C_M_NO_RD_ACK: u16 = 0x0800; /* if I2C_FUNC_PROTOCOL_MANGLING */
const I2C_M_IGNORE_NAK: u16 = 0x1000; /* if I2C_FUNC_PROTOCOL_MANGLING */
const I2C_M_REV_DIR_ADDR: u16 = 0x2000; /* if I2C_FUNC_PROTOCOL_MANGLING */
const I2C_M_NOSTART: u16 = 0x4000; /* if I2C_FUNC_NOSTART */
const I2C_M_STOP: u16 = 0x8000; /* if I2C_FUNC_PROTOCOL_MANGLING */ 

const I2C_RDRW_IOCTL_MAX_MSGS: u8 = 42;
const I2C_MAX_LEN: usize = 8192; // Magic value from i2cdev.c

#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
struct Message {
    addr: u16,
    flags: u16,
    len: u16,
    buffer: *const u8,
}

struct Factory {
    file: File,
    addr: u16,
}

impl Factory {
    pub fn new(device: &str, addr: u16) -> Result<Self, io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/i2c-0")?;

        let factory = Self {
            file,
            addr,
        };

        Ok(factory)
    }

    /// Send transaction to i2c hardware. Note that this function will panic!()
    /// if you provide more than `I2C_MAX_LEN` messages to save from difficult
    /// troubleshooting later.
    pub fn send(&self, messages: &[Message]) -> Result<(), nix::Error> {
        // TODO: Create a custom error type to avoid the panic!
        // TODO: Differentiate EBADF error on the dev tree vs. i2c slave device
        if messages.len() > I2C_RDRW_IOCTL_MAX_MSGS {
            panic!("Linux only allows {} message per transaction", I2C_MAX_LEN);
        }

        let i2c_data = IoctlData {
            messages: messages.as_ptr(),
            count: messages.len() as i32,
        };

        unsafe {
            ioctl::i2c_rdwr(self.file.as_raw_fd(), &i2c_data)?;
        }

        Ok(())
    }
}

impl Factory {
    pub fn custom(&self, data: &[u8], address: u16, flags: u16) -> Message {
        // TODO: Find a better way to inform users of too many messages than killing their app
        if data.len() > I2C_MAX_LEN {
            panic!("Tried to pack a message greater than {}", I2C_MAX_LEN);
        } else {
            Message {
                addr: self.addr,
                flags,
                len: data.len() as u16,
                buffer: data.as_ptr(),
            }
        }
    }

    pub fn read(&self, data: &[u8]) -> Message {
        self.custom(data, self.addr, I2C_M_RD)
    }

    pub fn write(&self, data: &[u8]) -> Message {
        self.custom(data, self.addr, 0)
    }
}

mod ioctl {
    use super::IoctlData;

    const I2C_RDWR: u32 = 0x0707;

    /// Call to I2C_RDWR. Not to be used by external folk
    ioctl_write_ptr_bad!(i2c_rdwr, I2C_RDWR, IoctlData);
}

#[repr(C)]
/// Structure for packing data to be sent to I2C_RDWR
pub struct IoctlData {
    // TODO: How the heck to I have a private ioctl so this can be hidden
    // away from users
    messages: *const Message,
    count: i32,
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn build_structure() {
        const MESSAGE: &'static [u8] = &[0x02];
        let mut data = [0u8; 1];

        let builder = Factory::new("/dev/i2c-0", 0x34).unwrap();
        let items = [
            builder.write(&MESSAGE),
            builder.read(&data),
        ];

        builder.send(&items);

        // Device specific
        assert!(data[0] == 0x04);
    }
}
