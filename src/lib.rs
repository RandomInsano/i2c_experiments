
#![allow(unused)]

/// Notes:
/// Implementation of i2c ioctls: https://github.com/torvalds/linux/blob/master/drivers/i2c/i2c-dev.c#L439

#[macro_use]
extern crate nix;
extern crate errno;

use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
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

const I2C_RDWR: u32 = 0x0707;

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

impl Message {
    pub fn read(data: &[u8]) -> Message {
        if data.len() > I2C_MAX_LEN {
            panic!("Tried to pack a message greater than {}", I2C_MAX_LEN);
        } else {
            Message {
                addr: 0x34,
                flags: I2C_M_RD,
                len: data.len() as u16,
                buffer: data.as_ptr(),
            }
        }
    }

    pub fn write(data: &[u8]) -> Message {
        if data.len() > I2C_MAX_LEN as usize {
            panic!("Tried to pack a message greater than {}", I2C_MAX_LEN);
        } else {
            Message {
                addr: 0x34,
                flags: 0,
                len: data.len() as u16,
                buffer: data.as_ptr(),
            }
        }
    }
}

ioctl_write_ptr_bad!(i2c_rdrw, I2C_RDWR, IoctlData);

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct IoctlData { 
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

        let mut items = [
            Message::write(&MESSAGE),
            Message::read(&data),
        ];

        let i2c_data = IoctlData {
            messages: items.as_ptr(),
            count: items.len() as i32,
        };

        let file_result = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/i2c-0");
        assert!(file_result.is_ok());

        let file = file_result.unwrap();
        let fd = file.as_raw_fd();

        println!("File descriptor: {}", fd);

        unsafe {
            println!();

            match i2c_rdrw(fd, &i2c_data) {
                Err(x) => {
                    println!("Error: {:?}", x);
                    panic!("ioclt failed!");
                },
                Ok(x) => {
                    println!("Ok: {:?}", x);
                    println!("Data: {:?}", data);
                },
            }
        }
    }
}
