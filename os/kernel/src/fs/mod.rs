pub mod sd;

use std::io;
use std::path::{Path, PathBuf};

use fat32::vfat::{Shared, VFat};
pub use fat32::traits;

use mutex::Mutex;
use self::sd::Sd;

#[derive(Debug)]
pub struct FileSystem(Mutex<Option<Shared<VFat>>>);

impl FileSystem {
    /// Returns an uninitialized `FileSystem`.
    ///
    /// The file system must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        FileSystem(Mutex::new(None))
    }

    /// Initializes the file system.
    ///
    /// # Panics
    ///
    /// Panics if the underlying disk or file sytem failed to initialize.
    pub fn initialize(&self) {
        *self.0.lock() = Some(VFat::from(Sd::new().unwrap()).unwrap());
    }
}

// FIXME: Implement `fat32::traits::FileSystem` for a useful type.
impl<'a> traits::FileSystem for &'a FileSystem {
    type File = <&'a Shared<VFat> as traits::FileSystem>::File;
    type Dir = <&'a Shared<VFat> as traits::FileSystem>::Dir;
    type Entry = <&'a Shared<VFat> as traits::FileSystem>::Entry;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        self.0.lock().as_ref().unwrap().open(path)
    }

    fn canonicalize<P: AsRef<Path>>(self, path: P) -> io::Result<PathBuf> {
        self.0.lock().as_ref().unwrap().canonicalize(path)
    }

    fn create_file<P: AsRef<Path>>(self, path: P) -> io::Result<Self::File> {
        self.0.lock().as_ref().unwrap().create_file(path)
    }

    fn create_dir<P>(self, path: P, parents: bool) -> io::Result<Self::Dir>
    where
        P: AsRef<Path>,
    {
        self.0.lock().as_ref().unwrap().create_dir(path, parents)
    }

    fn rename<P, Q>(self, from: P, to: Q) -> io::Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        self.0.lock().as_ref().unwrap().rename(from, to)
    }

    fn remove<P: AsRef<Path>>(self, path: P, children: bool) -> io::Result<()> {
        self.0.lock().as_ref().unwrap().remove(path, children)
    }
}
