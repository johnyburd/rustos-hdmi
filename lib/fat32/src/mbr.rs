use core::{fmt, mem};
use shim::const_assert_size;
use shim::io;

use crate::traits::BlockDevice;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CHS {
    head: u8,
    sector_plus_two_bits_of_cylinder: u8,
    the_rest_of_cylinder: u8,
}


impl fmt::Debug for CHS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CHS")
            .field("Head", &self.head)
            .field("Sector (sort of)", &self.sector_plus_two_bits_of_cylinder)
            .field("Cylinder (sort of)", &self.the_rest_of_cylinder)
            .finish()
    }
}

const_assert_size!(CHS, 3);

#[repr(C, packed)]
pub struct PartitionEntry {
    boot_indicator: u8,
    starting_chs: CHS,
    partition_type: u8,
    ending_chs: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32,
}

impl PartitionEntry {
    pub fn is_fat32(&self) -> bool {
        self.partition_type == 0xB || self.partition_type == 0xC
    }
}

impl fmt::Debug for PartitionEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Partition Entry")
            .field("Boot Indicator", &self.boot_indicator)
            .field("Starting CHS", &self.starting_chs)
            .field("Partition Type", &self.partition_type)
            .field("Ending CHS", &self.ending_chs)
            .field("Relative Sector", &self.relative_sector)
            .field("Total Sectors", &self.total_sectors)
            .finish()
    }
}

const_assert_size!(PartitionEntry, 16);

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    bootstrap: [u8; 436],
    disk_id: [u8; 10],
    pub partition_table: [PartitionEntry; 4],
    signature: [u8; 2],
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MBR")
            .field("MBR Bootstrap", &"you don't want me to print this")
            .field("Disk ID", &self.disk_id)
            .field("Partition Table", &self.partition_table)
            .field("Signature", &self.signature)
            .finish()
    }
}


const_assert_size!(MasterBootRecord, 512);

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut buf = [0u8; 512];
        device.read_sector(0, &mut buf).map_err(|e| { Error::Io(e) })?;
        let mbr = unsafe { mem::transmute::<[u8; 512], MasterBootRecord>(buf) };

        if mbr.signature != [0x55, 0xAA] {
            return Err(Error::BadSignature);
        }

        for (i, tab) in mbr.partition_table.iter().enumerate() {
            if !(tab.boot_indicator == 0 || tab.boot_indicator == 0x80) {
                return Err(Error::UnknownBootIndicator(i as u8));
            }
        }

        Ok(mbr)
    }
}
