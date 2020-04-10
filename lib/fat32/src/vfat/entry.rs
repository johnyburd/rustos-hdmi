use crate::traits;
use crate::vfat::{Dir, File, Metadata, VFatHandle};
use core::fmt;

// You can change this definition if you want
#[derive(Debug)]
pub enum Entry<HANDLE: VFatHandle> {
    EntryFile(File<HANDLE>),
    EntryDir(Dir<HANDLE>),
}

// Implement any useful helper methods on `Entry`.

// Implement `traits::Entry` for `Entry`.
impl<HANDLE: VFatHandle> traits::Entry for Entry<HANDLE> {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Metadata = Metadata;

    fn name(&self) -> &str {
        match self {
            &Entry::EntryFile(ref f) => &f.name(),
            &Entry::EntryDir(ref d) => &d.name(),
        }
    }

    fn metadata(&self) -> &Self::Metadata {
        match self {
            &Entry::EntryFile(ref f) => &f.metadata(),
            &Entry::EntryDir(ref d) => &d.metadata(),
        }
    }

    fn as_file(&self) -> Option<&Self::File> {
        match self {
            Entry::EntryFile(ref f) => Some(f),
            _ => None,
        }
    }

    fn as_dir(&self) -> Option<&Self::Dir> {
        match self {
            Entry::EntryDir(ref d) => Some(d),
            _ => None,
        }
    }

    fn into_file(self) -> Option<Self::File> {
        match self {
            Entry::EntryFile(f) => Some(f),
            _ => None,
        }
    }
    fn into_dir(self) -> Option<Self::Dir> {
        match self {
            Entry::EntryDir(d) => Some(d),
            _ => None,
        }
    }
}
