use volatile::prelude::*;
use volatile::{ReadVolatile, Reserved, Volatile, WriteVolatile};
use core::time::Duration;
use core::cmp::min;
use crate::common::MBOX_BASE;
use crate::timer::spin_sleep;

#[repr(C)]
struct Registers {
    READ: ReadVolatile<u32>,
    __reserved_0: [u32; 5], 
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
    pub mailbox: [Volatile<u32>; 36],
}

impl MailBox {

    pub fn new() -> MailBox {
        let mut mailbox: [Volatile<u32>; 36] = unsafe { core::mem::zeroed() };
        MailBox {
            registers: unsafe { &mut *(MBOX_BASE as *mut Registers) },
            mailbox: mailbox,
        }
    }

    pub fn mbox_call(&mut self, ch: u8) -> Result<(), ()> {
        let mut addr = &self.mailbox[0] as *const _ as usize;
        let addr = (addr & !0xF);
        let addr2 = (addr | (ch as usize & 0xF)) as u32;

        while self.registers.STATUS.read() & StatusTags::FULL as u32 != 0 {
            spin_sleep(Duration::from_micros(1));
        }

        self.registers.WRITE.write(addr2);

        loop {
            while self.registers.STATUS.read() & StatusTags::EMPTY as u32 != 0 {
                spin_sleep(Duration::from_micros(1));
            }

            if addr2 == self.registers.READ.read() {
                if self.mailbox[1].read() == StatusTags::FULL as u32 {
                    return Ok(())
                } else {
                    return Err(())
                }
            }
            return Ok(());
        }
    }
}

impl core::fmt::Debug for MailBox {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}
