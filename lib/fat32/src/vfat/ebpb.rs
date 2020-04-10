use core::{fmt, mem};
use shim::const_assert_size;

use crate::traits::BlockDevice;
use crate::vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    jmp: [u8; 3],
    oem_id: u64,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fats: u8,
    max_dir_entries: u16,
    total_logical_sectors: u16,
    fat_id: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
    total_logical_sectors_ovf: u32,

    sectors_per_fat_ovf: u32,
    flags: u16,
    fat_version: u16,
    pub root_cluster: u32,
    fsinfo_sector: u16,
    backup_boot_sector: u16,
    reserved: [u8; 12],
    drive_number: u8,
    windows_nt_flags: u8,
    signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    system_id_string: u64,
    boot_code: [u8; 420],
    bootable_partition_signature: u16,
}

const_assert_size!(BiosParameterBlock, 512);

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let mut buf = [0u8; 512];
        device.read_sector(sector, &mut buf).map_err(|e| { Error::Io(e) })?;
        let ebpb = unsafe { mem::transmute::<[u8; 512], BiosParameterBlock>(buf) };

        //if !(ebpb.signature == 0x28 || ebpb.signature == 0x29) {
            //return Err(Error::BadSignature);
        //}

        if ebpb.bootable_partition_signature != 0xAA55 {
            return Err(Error::BadSignature);
        }

        Ok(ebpb)
    }


    pub fn sectors_per_fat(&self) -> u32 {
        //to_u32(self.sectors_per_fat_ovf)
        self.sectors_per_fat_ovf
    }

    pub fn total_logical_sectors(&self) -> u64 {
        self.total_logical_sectors_ovf as u64
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BPB")
            .field("jmp", &self.jmp)
            .field("oem_id", &self.oem_id)
            .field("bytes_per_sector", &self.bytes_per_sector)
            .field("sectors_per_cluster", &self.sectors_per_cluster)
            .field("reserved_sectors", &self.reserved_sectors)
            .field("fats", &self.fats)
            .field("max_dir_entries", &self.max_dir_entries)
            .field("total_logical_sectors", &self.total_logical_sectors)
            .field("fat_id", &self.fat_id)
            .field("sectors_per_fat", &self.sectors_per_fat)
            .field("sectors_per_track", &self.sectors_per_track)
            .field("heads", &self.heads)
            .field("hidden_sectors", &self.hidden_sectors)
            .field("total_logical_sectors_ovf", &self.total_logical_sectors_ovf)
            .field("sectors_per_fat_ovf", &self.sectors_per_fat_ovf)
            .field("flags", &self.flags)
            .field("fat_version", &self.fat_version)
            .field("root_cluster", &self.root_cluster)
            .field("fsinfo_sector", &self.fsinfo_sector)
            .field("backup_boot_sector", &self.backup_boot_sector)
            .field("reserved", &self.reserved)
            .field("drive_number", &self.drive_number)
            .field("windows_nt_flags", &self.windows_nt_flags)
            .field("signature", &self.signature)
            .field("volume_id", &self.volume_id)
            .field("volume_label", &self.volume_label)
            .field("system_id_string", &self.system_id_string)
            //.field("boot_code", &self.boot_code)
            .field("bootable_partition_signature", &self.bootable_partition_signature)
            .finish()
    }
}
