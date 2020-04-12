use volatile::{Volatile};
use crate::mbox::MailBox;
use core::mem::{size_of, transmute};
use core::ptr::{copy_nonoverlapping, write_volatile};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
struct AllocateRes {
    fb_addr: usize,
    fb_size: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
struct ScreenSize {
    width: u32,
    height: u32,
}

#[repr(C)]
union ValueBuffer {
    fb_allocate_align: u32,
    fb_allocate_res: AllocateRes,
    fb_screen_size: ScreenSize,
    fb_bits_per_pixel: u32,
    fb_bytes_per_row: u32,
    null: (),
}

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

enum ReqRespCode {
    REQUEST = 0,
    SUCCESS = 0x80000000,
    ERROR = 0x80000001,
}

struct PropertyMessageTag {
    proptag: PropertyTag,
    value_buffer: ValueBuffer,
}

pub struct FrameBuffer {
    width: u32,
    height: u32,
    depth: u32,
    mailbox: MailBox,
}

pub struct PropertyMessageBuffer {
    size: u32,
    code: u32,
    tags: (),
}

fn get_value_buffer_len(tag: &PropertyMessageTag) -> usize {
    match tag.proptag {
        PropertyTag::FB_ALLOCATE_BUFFER => 8,
        PropertyTag::FB_GET_PHYSICAL_DIMENSIONS => 8,
        PropertyTag::FB_SET_PHYSICAL_DIMENSIONS => 8,
        PropertyTag::FB_GET_VIRTUAL_DIMENSIONS => 8,
        PropertyTag::FB_SET_VIRTUAL_DIMENSIONS => 8,
        PropertyTag::FB_GET_BITS_PER_PIXEL => 4,
        PropertyTag::FB_SET_BITS_PER_PIXEL => 4,
        PropertyTag::FB_GET_BYTES_PER_ROW => 4,
        PropertyTag::FB_RELESE_BUFFER => 0,
        _ => 0,
    }
}

fn send_messages(messages: &[PropertyMessageTag]) -> Result<(), ()> {
    let mut bufsize: usize = 0;
    let mut i = 0;
    let mut bufpos: u8 = 0;

    while messages[i].proptag != PropertyTag::NULL_TAG {
        bufsize += get_value_buffer_len(&messages[i]) + 3 * size_of::<u32>();
        i += 1;
    }

    bufsize += 3 * size_of::<u32>();

    //let mut buf: [u16; (bufsize / 2) + (bufsize % 2)] = core::mem::zeroed()
    let mut buf: [u16; 50] = [0; 50];
    //let ptr = &mut buf[0] as *mut Volatile<u8>;
    let ptr = unsafe { transmute::<&mut u16, *mut u32>(&mut buf[0]) };


    //let ptr = unsafe { alloc(Layout::from_size_align(bufsize, 16)) as *mut PropertyMessageBuffer };

    unsafe {

        write_volatile(ptr, bufsize as u32);

        write_volatile(ptr.offset(1), ReqRespCode::REQUEST as u32);
    }

    i = 0;
    let ptr = unsafe { transmute::<*mut u32, *mut u8>(ptr) };
    while messages[i].proptag != PropertyTag::NULL_TAG {
        let len = get_value_buffer_len(&messages[i]);
        unsafe {
            write_volatile(ptr.offset(bufpos as isize + 8), messages[i].proptag as u8);
            write_volatile(ptr.offset(bufpos as isize + 8 + 1), len as u8);
            write_volatile(ptr.offset(bufpos as isize + 8 + 2), 0u8);
            let trash = transmute::<*mut u8, *mut ValueBuffer>(ptr.offset(bufpos as isize + 8 + 3));
            copy_nonoverlapping(&messages[i].value_buffer as *const ValueBuffer, trash, len);
        }
        bufpos += 3 + (len as u8 / 4);
        i += 1;
    }

    // END TAG OR SMTH
    Ok(())
}

impl FrameBuffer {

    pub fn new(width: u32, height: u32, depth: u32) -> FrameBuffer {
        let mut mailbox = MailBox::new();

        let size = ScreenSize { width: width, height: height };

        let messages = [
            PropertyMessageTag {
                proptag: PropertyTag::FB_SET_PHYSICAL_DIMENSIONS,
                value_buffer: ValueBuffer { fb_screen_size: size }
            },
            PropertyMessageTag {
                proptag: PropertyTag::FB_SET_VIRTUAL_DIMENSIONS,
                value_buffer: ValueBuffer { fb_screen_size: size }
            },
            PropertyMessageTag {
                proptag: PropertyTag::FB_SET_BITS_PER_PIXEL,
                value_buffer: ValueBuffer { fb_bits_per_pixel: depth }
            },
            PropertyMessageTag {
                proptag: PropertyTag::NULL_TAG,
                value_buffer: ValueBuffer { fb_bits_per_pixel: 0 }
            },
        ];

        FrameBuffer {
            width: width,
            height: height,
            depth: depth,
            mailbox: mailbox,
        }
    }

    pub fn show_picture(&self) -> () {
    }
}
