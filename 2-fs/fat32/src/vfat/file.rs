use std::cmp::{max, min};
use std::io::{self, SeekFrom};

use traits;
use vfat::{Cluster, Metadata, Shared, Status, VFat};
use std::fmt;

pub struct File {
    // FIXME: Fill me in.
    pub(super) cluster: Cluster,
    pub(super) fs: Shared<VFat>,
    pub(super) short_name: String,
    pub(super) long_name: String,
    pub(super) metadata: Metadata,
    pub(super) file_size: u32,
    pub(super) current_offset: u32,
    pub(super) current_cluster: Option<Cluster>,
    pub(super) bytes_per_cluster: u32,
}

// FIXME: Implement `traits::File` (and its supertraits) for `File`.
impl traits::File for File {
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!()
    }

    fn size(&self) -> u64 {
        self.file_size as u64
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read_size = min(buf.len(), (self.file_size - self.current_offset) as usize);
        let mut current_offset_in_cluster = (self.current_offset % self.bytes_per_cluster) as usize;
        let mut fs = self.fs.borrow_mut();
        let mut rest_size = read_size;
        let mut current_cluster = self.current_cluster;
        let mut buffer_offset = 0;
        while rest_size > 0 {
            let newly_read_size = fs.read_cluster(
                current_cluster.unwrap(),
                current_offset_in_cluster,
                &mut buf[buffer_offset..read_size],
            )?;
            if newly_read_size == self.bytes_per_cluster as usize - current_offset_in_cluster {
                // read all content of current cluster
                match fs.fat_entry(current_cluster.unwrap())?.status() {
                    Status::Eoc(_) => current_cluster = None,
                    Status::Data(next_cluster) => {
                        current_cluster = Some(next_cluster);
                    }
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid cluster chain",
                        ))
                    }
                }
            }
            buffer_offset += newly_read_size;
            rest_size -= newly_read_size;
            current_offset_in_cluster = 0;
        }
        self.current_offset += read_size as u32;
        self.current_cluster = current_cluster;
        Ok(read_size)
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}

impl io::Seek for File {
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
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_offset = match pos {
            SeekFrom::Start(start) => start as u32,
            SeekFrom::Current(offset) => self.current_offset.wrapping_add(offset as u32),
            SeekFrom::End(offset) => self.file_size.wrapping_add(offset as u32),
        };
        if new_offset >= self.file_size {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek position",
            ))
        } else {
            let mut current_cluster = self.cluster;
            let mut fs = self.fs.borrow_mut();
            for _ in 0..(new_offset / self.bytes_per_cluster) {
                match fs.fat_entry(current_cluster)?.status() {
                    Status::Data(next_cluster) => {
                        current_cluster = next_cluster;
                    }
                    _ => unimplemented!(),
                }
            }
            self.current_cluster = Some(current_cluster);
            self.current_offset = new_offset;
            Ok(self.current_offset as u64)
        }
    }
}

impl File {
    pub fn name(&self) -> &str {
        if self.long_name.len() > 0 {
            self.long_name.as_str()
        } else {
            self.short_name.as_str()
        }
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("File")
            .field("short_name", &self.short_name)
            .field("long_name", &self.long_name)
            .field("cluster", &self.cluster)
            .field("metadata", &self.metadata)
            .field("file_size", &self.file_size)
            .finish()
    }
}
