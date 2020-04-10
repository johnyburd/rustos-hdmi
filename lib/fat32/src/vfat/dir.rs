use alloc::string::String;
use alloc::vec::{ Vec, IntoIter };

use shim::const_assert_size;
use shim::ffi::OsStr;
use shim::io;
use shim::newioerr;

use crate::traits;
use crate::util::VecExt;
use crate::vfat::{Attributes, Date, Metadata, Time, Timestamp};
use crate::vfat::{Cluster, Entry, File, VFatHandle};


#[derive(Debug)]
pub struct Dir<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub name: String,
    pub start_cluster: Cluster,
    pub metadata: Metadata,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    name: [u8; 8],
    extension: [u8; 3],
    attr: Attributes,
    windows_nt_reserved: u8,
    ctime_tenths: u8,
    ctime: Time,
    cdate: Date,
    adate: Date,
    cluster_num_high: u16,
    mtime: Time,
    mdate: Date,
    cluster_num_low: u16,
    file_size: u32,
}

impl VFatRegularDirEntry {
    pub fn cluster_number(&self) -> Cluster {
        Cluster::from((self.cluster_num_high as u32) << 16 | self.cluster_num_low as u32)
    }

    pub fn metadata(&self) -> Metadata {
        Metadata {
            attr: self.attr,
            ctime: Timestamp {
                time: self.ctime,
                date: self.cdate,
            },
            atime: Timestamp {
                time: Time::from(0),
                date: self.adate,
            },
            mtime: Timestamp {
                time: self.mtime,
                date: self.mdate,
            },
        }
    }
}

const_assert_size!(VFatRegularDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    sequence: u8,
    name1: [u16; 5],
    attr: Attributes,
    zero_type: u8,
    checksum: u8,
    name2: [u16; 6],
    zeros: u16,
    name3: [u16; 2],
}

const_assert_size!(VFatLfnDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    sequence: u8,
    unknown: [u16; 5],
    attr: Attributes,
    name2: [u16; 10],
}

const_assert_size!(VFatUnknownDirEntry, 32);

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl<HANDLE: VFatHandle> Dir<HANDLE> {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry<HANDLE>> {
        use traits::Dir;
        use traits::Entry;

        //TODO this
        let name_str = name.as_ref()
                           .to_str()
                           .ok_or(io::Error::new(io::ErrorKind::InvalidInput, "name contains invalid UTF-8 characters"))?;

        self.entries()?.find(|entry| entry.name().eq_ignore_ascii_case(name_str))
                       .ok_or(io::Error::new(io::ErrorKind::NotFound, "name not found"))
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct DirEntryIter<HANDLE: VFatHandle> {
    entries: IntoIter<VFatDirEntry>,
    vfat: HANDLE,
}


impl<HANDLE: VFatHandle> Iterator for DirEntryIter<HANDLE> {
    type Item = Entry<HANDLE>;
    fn next(&mut self) -> Option<Self::Item> {
        const lfn_len: usize = 13;
        let mut lfn = [0u16; 31 * lfn_len];
        let mut is_lfn = false;

        for ref entry in self.entries.by_ref() {
            let unknown = unsafe { entry.unknown };
            match unknown.sequence {
                0 => return None,
                0xe5 => continue,
                _ => (),
            };

            if unknown.attr.lfn() {
                let entry = unsafe { entry.long_filename };
                is_lfn = true;
                let entry_pos = ((entry.sequence & 0x1f) - 1) as usize;

                //lfn.copy_from_slice(&entry.name1);
                //lfn.copy_from_slice(&entry.name2);
                //lfn.copy_from_slice(&entry.name3);
                lfn[entry_pos * lfn_len      ..entry_pos * lfn_len + 5 ].copy_from_slice(&entry.name1);
                lfn[entry_pos * lfn_len + 5  ..entry_pos * lfn_len + 11].copy_from_slice(&entry.name2);
                lfn[entry_pos * lfn_len + 11 ..entry_pos * lfn_len + 13].copy_from_slice(&entry.name3);
            } else {
                let entry = unsafe { entry.regular };
                let name = if !is_lfn {
                    let mut name = entry.name.clone();
                    let name = ::core::str::from_utf8(&name).ok()?.trim_end();
                    let extension = ::core::str::from_utf8(&entry.extension).ok()?.trim_end();

                    let mut name = String::from(name);
                    if extension.len() > 0 {
                        name.push_str(".");
                        name.push_str(extension);
                    }
                    name
                } else {
                    let len = lfn.iter().position(|&c| c == 0x0000 || c == 0xffff).unwrap_or_else(||lfn.len());
                    String::from_utf16(&lfn[..len]).ok()?
                };

                //println!("name {} ", name);
                return if entry.attr.directory() {
                    Some(Entry::EntryDir(Dir {
                        name: name,
                        start_cluster: entry.cluster_number(),
                        vfat: self.vfat.clone(),
                        metadata: entry.metadata(),
                    }))
                } else {
                    Some(Entry::EntryFile(File {
                        name: name,
                        start_cluster: entry.cluster_number(),
                        vfat: self.vfat.clone(),
                        metadata: entry.metadata(),
                        size: entry.file_size,
                        cursor: 0,
                    }))
                };
            }
        }
        None
    }
    /*fn next(&mut self) -> Option<Self::Item> {
            // TODO
        //println!("NEXT");
        let mut lfn: Vec<u16> = Vec::new();

        for entry in self.entries.as_slice().iter() {
            let unknown = unsafe { entry.unknown };
            match unknown.sequence {
                0 => return None,
                0xe5 => continue,
                _ => (),


            if unknown.attr.lfn() {
                let entry = unsafe { entry.long_filename };
                lfn.copy_from_slice(&entry.name1);
                lfn.copy_from_slice(&entry.name2);
                lfn.copy_from_slice(&entry.name3);

            } else {
                let entry = unsafe { entry.regular };

                let name = if lfn.len() > 0 {
                    let name = ::core::str::from_utf8(&entry.name).ok()?.trim_end();
                    let ext = ::core::str::from_utf8(&entry.extension).ok()?.trim_end();
                    let mut name_str = String::from(name);
                    if ext.len() > 0 {
                        name_str.push_str(".");
                        name_str.push_str(ext);
                    }
                    name_str
                } else {
                    String::from_utf16(&lfn).ok()?
                };

                println!("name {} ", name);

                return if entry.attr.directory() {
                    Some(Entry::EntryDir(Dir {
                        name: name,
                        start_cluster: entry.cluster_number(),
                        vfat: self.vfat.clone(),
                        metadata: entry.metadata(),
                    }))
                } else {
                    Some(Entry::EntryFile(File {
                        name: name,
                        start_cluster: entry.cluster_number(),
                        vfat: self.vfat.clone(),
                        metadata: entry.metadata(),
                        size: entry.file_size,
                        cursor: 0,
                    }))
                }

            }
        }
        None
    }*/
}


    //Implement `trait::Dir` for `Dir`.
impl<HANDLE: VFatHandle> traits::Dir for Dir<HANDLE> {
    type Entry = Entry<HANDLE>;

    type Iter = DirEntryIter<HANDLE>;

    fn entries(&self) -> io::Result<Self::Iter> {
        let mut buf = Vec::new();
        self.vfat.lock(|vfat| vfat.read_chain(self.start_cluster, &mut buf).expect("error reading chain"));

        Ok(DirEntryIter {
            entries: unsafe { buf.cast() }.into_iter(),
            vfat: self.vfat.clone()
        })

    }
}
