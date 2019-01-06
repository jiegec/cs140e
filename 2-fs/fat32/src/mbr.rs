use std::{fmt, io};

use traits::BlockDevice;
use std::mem::transmute_copy;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct CHS {
    // FIXME: Fill me in.
    head: u8,
    sector_and_high_cylinder: u8,
    low_cylinder: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct PartitionEntry {
    // FIXME: Fill me in.
    boot_indicator: u8,
    first_sector: CHS,
    partition_type: u8, // 0xB or 0xC for FAT32
    last_sector: CHS,
    first_sector_lba: u32,
    num_sectors: u32,
}

/// The master boot record (MBR).
// In some platforms, a u32 must be aligned to 4-byte boundary
#[repr(C, align(512))]
pub struct MasterBootRecord {
    // FIXME: Fill me in.
    __r1: [u8; 436],
    unique_disk_id: [u8; 10],
    partitions: [PartitionEntry; 4],
    magic: u16,
}

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
        let size = device.read_sector(0, &mut buf)?;
        if size != 512 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unable to read 512 bytes of MBR",
            ).into());
        } else {
            let result: MasterBootRecord = unsafe { transmute_copy(&buf) };
            if result.magic != 0xaa55 {
                return Err(Error::BadSignature);
            }

            for i in 0..4usize {
                if result.partitions[i].boot_indicator != 0x00
                    && result.partitions[i].boot_indicator != 0x80
                {
                    return Err(Error::UnknownBootIndicator(i as u8));
                }
            }
            return Ok(result);
        }
    }

    pub fn first_vfat_partition_lba(&self) -> Option<u32> {
        for i in 0..4 {
            if self.partitions[i].partition_type == 0xB || self.partitions[i].partition_type == 0xC
            {
                return Some(self.partitions[i].first_sector_lba);
            }
        }
        None
    }
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MasterBootRecord")
            .field("partitions", &self.partitions)
            .finish()
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl fmt::Debug for CHS {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("CHS").finish()
    }
}
