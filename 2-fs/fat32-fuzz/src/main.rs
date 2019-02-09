#[macro_use]
extern crate afl;
extern crate fat32;

use fat32::traits::BlockDevice;
use fat32::vfat::VFat;
use std::io::Cursor;

fn main() {
    fuzz!(|data: &[u8]| {
        let vec = Vec::from(data);
        if let Ok(vfat) = VFat::from(Cursor::new(vec)) {

        }
    });
}
