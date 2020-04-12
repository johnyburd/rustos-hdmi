use volatile::prelude::*;
use volatile::{Volatile};
use crate::mbox::MailBox;

#[derive(Debug, Copy, Clone, PartialEq)]
enum PropertyTag {
    NULL_TAG = 0,
    FB_ALLOCATE_BUFFER = 0x00040001,
    FB_RELESE_BUFFER = 0x00048001,
    FB_GET_PHYSICAL_DIMENSIONS = 0x00040003,
    FB_SET_PHYSICAL_DIMENSIONS = 0x00048003,
    FB_GET_VIRTUAL_DIMENSIONS = 0x00040004,
    FB_SET_VIRTUAL_DIMENSIONS = 0x00048004,
    FB_GET_BITS_PER_PIXEL = 0x00040005,
    FB_SET_BITS_PER_PIXEL = 0x00048005,
    FB_GET_BYTES_PER_ROW = 0x00040008,
}

#[derive(Debug)]
pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub isrgb: u32,
    pub te: u32,
    pub tn: u32,
    pub t: u32,
    pub to: u32,
    pub mailbox: MailBox,
}

impl FrameBuffer {

    pub fn new(width: u32, height: u32, depth: u32) -> FrameBuffer {
        let mut mailbox = MailBox::new();

        mailbox.mailbox[0].write(35*4);
        mailbox.mailbox[1].write(0);

        mailbox.mailbox[2].write(0x48003);  //set phy wh
        mailbox.mailbox[3].write(8);
        mailbox.mailbox[4].write(8);
        mailbox.mailbox[5].write(1024);         //FrameBufferInfo.width
        mailbox.mailbox[6].write(768);          //FrameBufferInfo.height

        mailbox.mailbox[7].write(0x48004);  //set virt wh
        mailbox.mailbox[8].write(8);
        mailbox.mailbox[9].write(8);
        mailbox.mailbox[10].write(1024);        //FrameBufferInfo.virtual_width
        mailbox.mailbox[11].write(768);         //FrameBufferInfo.virtual_height

        mailbox.mailbox[12].write(0x48009); //set virt offset
        mailbox.mailbox[13].write(8);
        mailbox.mailbox[14].write(8);
        mailbox.mailbox[15].write(0);           //FrameBufferInfo.x_offset
        mailbox.mailbox[16].write(0);           //FrameBufferInfo.y.offset

        mailbox.mailbox[17].write(0x48005); //set depth
        mailbox.mailbox[18].write(4);
        mailbox.mailbox[19].write(4);
        mailbox.mailbox[20].write(32);          //FrameBufferInfo.depth

        mailbox.mailbox[21].write(0x48006); //set pixel order
        mailbox.mailbox[22].write(4);
        mailbox.mailbox[23].write(4);
        mailbox.mailbox[24].write(1);           //RGB, not BGR preferably

        mailbox.mailbox[25].write(0x40001); //get framebuffer, gets alignment on request
        mailbox.mailbox[26].write(8);
        mailbox.mailbox[27].write(8);
        mailbox.mailbox[28].write(4096);        //FrameBufferInfo.pointer
        mailbox.mailbox[29].write(0);           //FrameBufferInfo.size

        mailbox.mailbox[30].write(0x40008); //get pitch
        mailbox.mailbox[31].write(4);
        mailbox.mailbox[32].write(4);
        mailbox.mailbox[33].write(0);           //FrameBufferInfo.pitch

        mailbox.mailbox[34].write(0);

        mailbox.mbox_call(8).expect("mbox call");
        //TODO convert to arm

        FrameBuffer {
            width: mailbox.mailbox[5].read(),
            height: mailbox.mailbox[6].read(),
            pitch: mailbox.mailbox[33].read(),
            isrgb: mailbox.mailbox[24].read(),
            te: mailbox.mailbox[28].read(),
            tn: mailbox.mailbox[29].read(),
            t: mailbox.mailbox[30].read(),
            to: mailbox.mailbox[31].read(),
            mailbox: mailbox,
        }
    }

    pub fn show_picture(&self) -> () {
    }
}
