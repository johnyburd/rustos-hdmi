use alloc::string::String;

use shim::io::{self, SeekFrom};

use crate::traits;
use crate::vfat::{Cluster, Metadata, VFatHandle};

#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub name: String,
    pub start_cluster: Cluster,
    pub metadata: Metadata,
    pub size: u32,
    pub cursor: u32,
}

impl<HANDLE: VFatHandle> File<HANDLE> {
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

// Implement `traits::File` (and its supertraits) for `File`.

impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }
    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}

impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut file_buf = vec!();
        self.vfat.lock(|vfat| vfat.read_chain(self.start_cluster, &mut file_buf))?;

        let len = ::core::cmp::min(buf.len(), (self.size - self.cursor) as usize);
        buf[..len].copy_from_slice(&file_buf[self.cursor as usize..(self.cursor as usize + len)]);
        self.cursor += len as u32;
        Ok(len)
    }
}

impl<HANDLE: VFatHandle> io::Seek for File<HANDLE> {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        unimplemented!("File::seek()")
    }
}

impl<HANDLE: VFatHandle> traits::File for File<HANDLE> {

    fn sync(&mut self) -> io::Result<()> {
        unimplemented!()
    }

    fn size(&self) -> u64 {
        self.size as u64
    }
}
