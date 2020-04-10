use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;
use alloc::string::String;

use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::Path;
use shim::path::{Component};

use crate::mbr::MasterBootRecord;
use crate::traits::{BlockDevice, FileSystem};
use crate::util::SliceExt;
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};
use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status, Metadata};

/// A generic trait that handles a critical section as a closure
pub trait VFatHandle: Clone + Debug + Send + Sync {
    fn new(val: VFat<Self>) -> Self;
    fn lock<R>(&self, f: impl FnOnce(&mut VFat<Self>) -> R) -> R;
}

#[derive(Debug)]
pub struct VFat<HANDLE: VFatHandle> {
    phantom: PhantomData<HANDLE>,
    device: CachedPartition,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    rootdir_cluster: Cluster,
}

impl<HANDLE: VFatHandle> VFat<HANDLE> {
    pub fn from<T>(mut device: T) -> Result<HANDLE, Error>
    where
        T: BlockDevice + 'static,
    {

        let mbr = MasterBootRecord::from(&mut device)?;
        let fat_part = mbr.partition_table.iter()
            .find(|part| part.is_fat32())
            .ok_or(Error::NotFound)?;
        let ebpb_sector = fat_part.relative_sector as u64;
        let ebpb = BiosParameterBlock::from(&mut device, ebpb_sector)?;
      //  println!("{:#?}", mbr);
    //    println!("{:#?}", ebpb);
        let fat_sector = ebpb_sector + ebpb.reserved_sectors as u64;
        let fat_sector = ebpb.reserved_sectors as u64;
        //println!("fat sector {}", fat_sector);
        let data_sector = fat_sector + (ebpb.fats as u64 * ebpb.sectors_per_fat() as u64);
        //println!("data sector {}", data_sector);
        let cached_part = CachedPartition::new(device, Partition {
                                                  start: ebpb_sector,
                                                  //num_sectors: fat_part.total_sectors as u64,
                                                  num_sectors: ebpb.total_logical_sectors(),
                                                  sector_size: ebpb.bytes_per_sector as u64});
        //println!("bps: {}", ebpb.bytes_per_sector);

        Ok(VFatHandle::new(VFat {
            phantom: PhantomData,
            device: cached_part,
            bytes_per_sector: ebpb.bytes_per_sector,
            sectors_per_cluster: ebpb.sectors_per_cluster,
            sectors_per_fat: ebpb.sectors_per_fat(),
            fat_start_sector: fat_sector,
            data_start_sector: data_sector,
            rootdir_cluster: Cluster::from(ebpb.root_cluster),
        }))


    }

    //  The following methods may be useful here:
    //
    //  * A method to read from an offset of a cluster into a buffer.
    //

    fn read_cluster(&mut self, cluster: Cluster, offset: usize, buf: &mut [u8]) -> io::Result<usize> {
        //println!("read_cluster starting {:?} offset {}", cluster, offset);
        let cluster_start = self.data_start_sector + cluster.get_offset() * self.sectors_per_cluster as u64;
        let first_sector = cluster_start + offset as u64;
        let max_sectors_in_buf = buf.len() as u64 / self.bytes_per_sector as u64;

        let last_sector = ::core::cmp::min(self.sectors_per_cluster as u64, max_sectors_in_buf) + cluster_start;

        let mut read = 0;
        //println!("reading {} sectors", last_sector - first_sector);
        for i in first_sector..last_sector {
            read += self.device.read_sector(i, &mut buf[read..])?;
        }
        //println!("read_cluster read {} bytes", read);
        Ok(read)
    }
    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
    pub fn read_chain(&mut self, start: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut read = 0;
        let mut cluster = start;

        loop {
            buf.resize(buf.len() + self.bytes_per_sector as usize * self.sectors_per_cluster as usize, 0);
            read += self.read_cluster(cluster, 0, &mut buf[read..])?;
            match self.fat_entry(cluster)?.status() {
                Status::Data(next) => cluster = next,
                Status::Eoc(_) => {
                    //println!("cluster read successfully");
                    return Ok(read);},
                _ => return Err(io::Error::new(io::ErrorKind::Other, "unexpected entry")),
            }
        }
    }
    //
    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {

        //println!("fat entry for {:#?}", cluster);


        let entries_per_sector = self.bytes_per_sector as usize / size_of::<FatEntry>();
        let cluster_start = cluster.get() as usize;
        let fat_entry_sec = (cluster_start / entries_per_sector) as u64;
        let sec_entry = cluster_start % entries_per_sector;
        let sec = self.device.get(self.fat_start_sector as u64 + fat_entry_sec)?;
        let entries: &[FatEntry] = unsafe { sec.cast() };
        //println!("{:?}", entries[sec_entry]);
        Ok(&entries[sec_entry])
    }
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Entry = Entry<HANDLE>;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {

        use crate::traits::Entry;
        use crate::vfat::Entry as EntryEnum;

        let mut entry = EntryEnum::EntryDir(Dir {
            vfat: self.clone(),
            name: String::from("/"),
            start_cluster: self.lock(|vfat| vfat.rootdir_cluster),
            metadata: Metadata::default(),
        });
        for compontent in path.as_ref().components() {
            if let Component::Normal(s) = compontent {
                entry = entry.as_dir().expect("entry as dir").find(s)?;
            }
        }
        //Err(io::Error::new(io::ErrorKind::Other, "hello"))
        Ok(entry)
    }
}
