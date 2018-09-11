
#![allow(unused)]

extern crate libc;

use libc::ioctl;
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::io::AsRawFd;

const I2C_M_RD: u16 = 0x0001; /* read data, from slave to master */
const I2C_M_TEN: u16 = 0x0010; /* this is a ten bit chip address */
const I2C_M_RECV_LEN: u16 = 0x0400; /* length will be first received byte */
const I2C_M_NO_RD_ACK: u16 = 0x0800; /* if I2C_FUNC_PROTOCOL_MANGLING */
const I2C_M_IGNORE_NAK: u16 = 0x1000; /* if I2C_FUNC_PROTOCOL_MANGLING */
const I2C_M_REV_DIR_ADDR: u16 = 0x2000; /* if I2C_FUNC_PROTOCOL_MANGLING */
const I2C_M_NOSTART: u16 = 0x4000; /* if I2C_FUNC_NOSTART */
const I2C_M_STOP: u16 = 0x8000; /* if I2C_FUNC_PROTOCOL_MANGLING */ 

const I2C_RDWR: u64 = 0x0707;
const I2C_RDRW_IOCTL_MAX_MSGS: u8 = 42;

#[repr(C)]
#[allow(non_camel_case_types)]
struct Message<'a> {
    addr: u16,
    flags: u16,
    len: u16,
    buffer: &'a [u8],
}

impl<'a> Message<'a> {
    pub fn new(data: &'a [u8]) -> Message<'a> {
        if data.len() > std::u16::MAX as usize {
            panic!("Tried to pack a message greater than {}", std::u16::MAX);
        } else {
            Message {
                addr: 0x1,
                flags: I2C_M_RD,
                len: data.len() as u16,
                buffer: data,
            }
        }
    }
}

#[repr(C)]
#[allow(non_camel_case_types)]
struct IoctlData<'a> { 
    messages: &'a [Message<'a>],
    count: i32,
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    /// This mostly exists to make sure I'm coding things properly. The length
    /// isn't something that's going to break.
    #[test]
    fn build_structure() {
        let items = [
            Message::new(&[0u8; 12]),
            Message::new(&[0u8; 13]),
            Message::new(&[0u8; 14]),
        ];

        let i2c_data = IoctlData {
            messages: &items,
            count: items.len() as i32,
        };

        let file_result = File::open("/dev/i2c-0");

        assert!(i2c_data.messages.len() == 3);
        assert!(file_result.is_ok());

        unsafe {
            ioctl(file_result.unwrap().as_raw_fd(), I2C_RDWR, &i2c_data);
        }
    }

    /*
    pub unsafe fn spi_read_mode(fd: c_int, data: *mut u8) -> Result<c_int> {
        let res = libc::ioctl(fd, ior!(SPI_IOC_MAGIC, SPI_IOC_TYPE_MODE, mem::size_of::<u8>()), data);
        Errno::result(res)
    }
    */
}
