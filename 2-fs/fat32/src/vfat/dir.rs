use std::mem::size_of;
use std::ffi::OsStr;
use std::char::decode_utf16;
use std::borrow::Cow;
use std::io;

use traits;
use util::VecExt;
use vfat::{Cluster, Entry, File, Shared, VFat};
use vfat::{Attributes, Date, Metadata, Time, Timestamp};
use std::fmt;
use std::str;

pub struct Dir {
    // FIXME: Fill me in.
    cluster: Cluster,
    fs: Shared<VFat>,
    short_name: String,
    long_name: String,
    pub(super) metadata: Metadata,
}

#[repr(C, align(32))]
#[derive(Debug, Copy, Clone)]
pub struct VFatRegularDirEntry {
    // FIXME: Fill me in.
    short_file_name: [u8; 8],
    short_file_extension: [u8; 3],
    metadata: Metadata,
    file_size: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct VFatLfnDirEntry {
    // FIXME: Fill me in.
    sequence_number: u8,
    name_characters: [u16; 5],
    attributes: Attributes,
    type_: u8,
    checksum_of_file_name: u8,
    name_characters_2: [u16; 6],
    always_zero: u16,
    name_characters_3: [u16; 2],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    // FIXME: Fill me in.
    status: u8,
    __r1: [u8; 10],
    attributes: Attributes,
    __r2: [u8; 20],
}

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl Dir {
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
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry> {
        use traits::{Dir, Entry};
        let lowercase_name = match name.as_ref().to_str() {
            Some(name_str) => name_str.to_lowercase(),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "name is not valid UTF-8 string",
                ))
            }
        };
        for entry in self.entries()? {
            if lowercase_name == entry.name().to_lowercase() {
                return Ok(entry);
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "entry not found"))
    }

    pub fn name(&self) -> &str {
        if self.long_name.len() > 0 {
            self.long_name.as_str()
        } else {
            self.short_name.as_str()
        }
    }

    pub(super) fn new_root(fs: &Shared<VFat>) -> Dir {
        let cluster = fs.borrow().root_dir_cluster;
        Dir {
            cluster: cluster,
            fs: fs.clone(),
            short_name: String::new(),
            long_name: String::new(),
            metadata: Metadata::default(),
        }
    }
}

// FIXME: Implement `trait::Dir` for `Dir`.
pub struct EntryIterator {
    data: Vec<u8>,
    current_index: usize,
    fs: Shared<VFat>,
    bytes_per_cluster: u32,
}

impl Iterator for EntryIterator {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let entries: *const VFatDirEntry = self.data.as_ptr() as *const VFatDirEntry;
        let mut long_file_name = [0u16; 260];
        while self.current_index * size_of::<VFatDirEntry>() < self.data.len() {
            let current_entry: &VFatDirEntry = unsafe {
                entries
                    .offset(self.current_index as isize)
                    .as_ref()
                    .unwrap()
            };
            let unknown_entry = unsafe { current_entry.unknown };
            if unknown_entry.status == 0x00 {
                // End of FAT
                return None;
            } else if unknown_entry.status == 0xE5 {
                // Deleted entry
                self.current_index += 1;
                continue;
            }

            // Normal entry,
            self.current_index += 1;
            if unknown_entry.attributes.lfn() {
                let lfn_entry = unsafe { current_entry.long_filename };
                let lfn_sequence_num = (lfn_entry.sequence_number & 0x1F) as usize - 1;

                if lfn_sequence_num <= 19 {
                    long_file_name[lfn_sequence_num * 13..lfn_sequence_num * 13 + 5]
                        .copy_from_slice(&lfn_entry.name_characters);
                    long_file_name[lfn_sequence_num * 13 + 5..lfn_sequence_num * 13 + 11]
                        .copy_from_slice(&lfn_entry.name_characters_2);
                    long_file_name[lfn_sequence_num * 13 + 11..lfn_sequence_num * 13 + 13]
                        .copy_from_slice(&lfn_entry.name_characters_3);
                }
            } else {
                let regular_entry = unsafe { current_entry.regular };
                let mut short_file_name = regular_entry.short_file_name.clone();
                if short_file_name[0] == 0x05 {
                    // 0x05 is used for real 0xE5 as first byte
                    short_file_name[0] = 0xE5;
                }
                let name = str::from_utf8(&short_file_name).unwrap().trim_right();
                let ext = str::from_utf8(&regular_entry.short_file_extension)
                    .unwrap()
                    .trim_right();
                let mut short_name = String::from(name);
                if ext.len() > 0 {
                    short_name.push_str(".");
                    short_name.push_str(ext);
                }
                let mut nul_byte_index = None;
                for (i, byte) in long_file_name.iter().enumerate() {
                    if *byte == 0 {
                        nul_byte_index = Some(i);
                        break;
                    }
                }
                let long_name = String::from_utf16(if let Some(len) = nul_byte_index {
                    &long_file_name[0..len]
                } else {
                    &long_file_name
                }).unwrap();
                if regular_entry.metadata.attributes.directory() {
                    return Some(Entry::Dir(Dir {
                        cluster: Cluster::from(regular_entry.metadata.first_cluster()),
                        fs: self.fs.clone(),
                        short_name,
                        long_name,
                        metadata: regular_entry.metadata,
                    }));
                } else {
                    return Some(Entry::File(File {
                        cluster: Cluster::from(regular_entry.metadata.first_cluster()),
                        fs: self.fs.clone(),
                        short_name,
                        long_name,
                        metadata: regular_entry.metadata,
                        file_size: regular_entry.file_size,
                        current_offset: 0,
                        current_cluster: Some(Cluster::from(
                            regular_entry.metadata.first_cluster(),
                        )),
                        bytes_per_cluster: self.bytes_per_cluster,
                    }));
                }
            }
        }
        None
    }
}

impl traits::Dir for Dir {
    type Entry = Entry;

    /// An type that is an iterator over the entries in this directory.
    type Iter = EntryIterator;

    /// Returns an interator over the entries in this directory.
    fn entries(&self) -> io::Result<Self::Iter> {
        let mut data = Vec::new();
        let bytes_per_cluster = {
            let mut fs_borrow = self.fs.borrow_mut();
            fs_borrow.read_chain(self.cluster, &mut data)?;
            fs_borrow.bytes_per_sector as u32 * fs_borrow.sectors_per_cluster as u32
        };

        Ok(EntryIterator {
            data: data,
            current_index: 0,
            fs: self.fs.clone(),
            bytes_per_cluster,
        })
    }
}

impl fmt::Debug for Dir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Dir")
            .field("short_name", &self.short_name)
            .field("long_name", &self.long_name)
            .field("cluster", &self.cluster)
            .field("metadata", &self.metadata)
            .finish()
    }
}
