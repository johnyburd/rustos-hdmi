use volatile::prelude::*;
use volatile::{ReadVolatile, Reserved, Volatile, WriteVolatile};
use core::time::Duration;
use core::cmp::min;
use crate::common::MBOX_BASE;
use crate::timer::spin_sleep;


#[repr(C)]
pub struct Message { msg: u32 }

impl Message {

    pub fn new(channel: u8, data: u32) -> Message {
        Message { msg: (channel & 0b1111) as u32 | data << 4 }
    }

    pub fn channel(&self) -> u8 {
        (self.msg & 0b1111) as u8
    }

    pub fn data(&self) -> u32 {
        self.msg >> 4
    }
}

#[repr(C)]
struct Registers {
    READ: ReadVolatile<u32>,
    POLL: ReadVolatile<u32>,
    SENDER: ReadVolatile<u32>,
    STATUS: ReadVolatile<u32>,
    CONFIG: ReadVolatile<u32>,
    WRITE: WriteVolatile<u32>,
}

enum StatusTags {
    FULL = 0x80000000,
    EMPTY = 0x40000000,
}

#[repr(C)]
pub struct MailBox {
    registers: &'static mut Registers,
    //mailbox: [Volatile<u32>; 36],
}

impl MailBox {

    pub fn new() -> MailBox {
        //let mut mailbox: [Volatile<u32>; 36] = unsafe { core::mem::zeroed() };
        MailBox {
            registers: unsafe { &mut *(MBOX_BASE as *mut Registers) },
            //mailbox: mailbox,
        }
    }

    pub fn recv(&self, ch: u8) -> Result<Message, ()> {

        while self.registers.STATUS.read() & StatusTags::EMPTY as u32 != 0 {
            spin_sleep(Duration::from_micros(1));
        }


        let msg = Message { msg: self.registers.READ.read() };
        if msg.channel() == ch {
            return Ok(msg);
        }

        Err(())
    }

    pub fn send(&mut self, msg: Message) -> () {

        while self.registers.STATUS.read() & StatusTags::FULL as u32 != 0 {
            spin_sleep(Duration::from_micros(1));
        }

        self.registers.WRITE.write(msg.msg);
    }
}
