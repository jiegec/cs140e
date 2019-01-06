use std::fmt;

use traits::BlockDevice;
use vfat::Error;
use util::SliceExt;
use std::io;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct BiosParameterBlock {
    // FIXME: Fill me in.
    jump_instruction: [u8; 3],
    oem_identifier: [u8; 8],
    pub(super) bytes_per_sector: u16,
    pub(super) sectors_per_cluster: u8,
    pub(super) reserved_sectors: u16,
    pub(super) number_of_fats: u8,
    max_directory_entries: u16, // always 0 in FAT32
    total_logical_sectors: u16, // use _2 in FAT32
    media_descriptor: u8,
    sectors_per_fat: u16, // use _2 in FAT32
    sectors_per_track: u16,
    number_of_heads: u16,
    hidden_sectors: u32,
    total_logical_sectors_2: u32,
    sectors_per_fat_2: u32,
    flags: u16,
    version: u16,
    pub(super) root_directory_cluster: u32,
    location_of_fs_information_sector: u16,
    location_of_backup_sector: u16,
    __r2: [u8; 12],
    physical_drive_number: u8,
    __r3: u8,
    signature: u8, // 0x28 or 0x29
    volume_serial_number: u32,
    volume_label: [u8; 11],
    fs_type: [u8; 8],
    __r4: [u8; 420],
    magic: u16,
}

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let mut buf = [0u8; 512];
        match device.read_sector(sector, &mut buf) {
            Ok(size) => {
                if size != 512 {
                    Err(Error::Io(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "unable to read 512 bytes of EBPB",
                    )))
                } else {
                    let result: &BiosParameterBlock = unsafe { &buf.cast()[0] };
                    if result.magic == 0xaa55 {
                        Ok(result.clone())
                    } else {
                        Err(Error::BadSignature)
                    }
                }
            }
            Err(err) => Err(Error::Io(err)),
        }
    }

    pub(super) fn sectors_per_fat(&self) -> u32 {
        if self.sectors_per_fat > 0 {
            self.sectors_per_fat as u32
        } else {
            self.sectors_per_fat_2
        }
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParameterBlock").finish()
    }
}
