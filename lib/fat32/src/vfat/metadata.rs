use core::fmt;

use alloc::string::String;

use crate::traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

impl Date {
    pub fn year(&self) -> usize {
        (self.0 as usize >> 9) + 1980
    }

    pub fn month(&self) -> u8 {
        ((self.0 & 0x1e0) >> 5) as u8
    }

    pub fn day(&self) -> u8 {
        self.0 as u8 & 0x1f
    }
}

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

impl Time {

    pub fn from(time: u16) -> Self {
        Time(time)
    }

    pub fn hour(&self) -> u8 {
        (self.0 >> 11) as u8
    }

    pub fn minute(&self) -> u8 {
        ((self.0 & 0x7e0) >> 5) as u8
    }

    pub fn second(&self) -> u8 {
        (self.0 as u8 & 0x1f) * 2
    }
}

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);

impl Attributes {
    pub fn read_only(&self) -> bool {
        self.0 & 0x1 != 0
    }

    pub fn hidden(&self) -> bool {
        self.0 & 0x2 != 0
    }

    pub fn system(&self) -> bool {
        self.0 & 0x4 != 0
    }

    pub fn volume_id(&self) -> bool {
        self.0 & 0x8 != 0
    }

    pub fn directory(&self) -> bool {
        self.0 & 0x10 != 0
    }

    pub fn archive(&self) -> bool {
        self.0 & 0x20 != 0
    }

    pub fn lfn(&self) -> bool {
        self.0 == 0xf
    }
}

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub date: Date,
    pub time: Time,
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub attr: Attributes,
    pub ctime: Timestamp,
    pub atime: Timestamp,
    pub mtime: Timestamp,
}

// Implement `traits::Timestamp` for `Timestamp`.

impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize {
        self.date.year()
    }

    fn month(&self) -> u8 {
        self.date.month()
    }

    fn day(&self) -> u8 {
        self.date.day()
    }

    fn hour(&self) -> u8 {
        self.time.hour()
    }

    fn minute(&self) -> u8 {
        self.time.minute()
    }

    fn second(&self) -> u8 {
        self.time.second()
    }
}

// Implement `traits::Metadata` for `Metadata`.

impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    fn read_only(&self) -> bool {
        self.attr.read_only()
    }

    fn hidden(&self) -> bool {
        self.attr.hidden()
    }

    fn created(&self) -> Self::Timestamp {
        self.ctime
    }

    fn accessed(&self) -> Self::Timestamp {
        self.atime
    }

    fn modified(&self) -> Self::Timestamp {
        self.mtime
    }
}

// Implement `fmt::Display` (to your liking) for `Metadata`.

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Metadata")
         .field("ro", &self.attr.read_only())
         .field("hidden", &self.attr.hidden())
         .field("created", &self.ctime)
         .field("accessed", &self.atime)
         .field("modified", &self.mtime)
         .finish()
    }
}
