use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::mem::transmute;
use traits::FileSystem;
use traits::Entry;
use traits::Dir;
use traits::File;
use util::SliceExt;
use util::VecExt;

#[test]
fn test_raspbian_img_vfat() {
    let file =
        fs::File::open("/Volumes/Data/raspbian/2017-11-29-raspbian-stretch-lite.img").unwrap();
    let shared_vfat = ::vfat::VFat::from(file).unwrap();
    {
        let mut vfat = shared_vfat.borrow_mut();
        let root_cluster = vfat.root_dir_cluster;
        let mut buffer = Vec::new();
        vfat.read_chain(root_cluster, &mut buffer).unwrap();
        println!("{:?}", *vfat);
    }

    let root = shared_vfat.open("/").unwrap();
    println!("{:?}", root);
    let root_dir = root.as_dir().unwrap();
    for entry in root_dir.entries().unwrap() {
        println!("{:?}", entry);
        if entry.is_dir() {
            for sub_entry in entry.as_dir().unwrap().entries().unwrap() {
                println!("{:?}", sub_entry);
            }
        }
    }
    let overlays = shared_vfat.open("/overlays/ads1015.dtbo").unwrap();
    println!("{:?}", overlays);
}

#[test]
fn test_raspbian_img_mbr() {
    let file =
        fs::File::open("/Volumes/Data/raspbian/2017-11-29-raspbian-stretch-lite.img").unwrap();
    let mbr = ::mbr::MasterBootRecord::from(file);
    println!("{:?}", mbr);
}

#[test]
fn test_raspbian_img_ebpb() {
    let mut file =
        fs::File::open("/Volumes/Data/raspbian/2017-11-29-raspbian-stretch-lite.img").unwrap();
    let mbr = ::mbr::MasterBootRecord::from(&mut file).unwrap();
    let fat_start_sector = mbr.first_vfat_partition_lba().unwrap() as u64;
    let ebpb = ::vfat::ebpb::BiosParameterBlock::from(&mut file, fat_start_sector).unwrap();
    println!("{:?}", ebpb);
}

#[test]
fn test_raspbian_img_config_txt() {
    let mut file =
        fs::File::open("/Volumes/Data/raspbian/2017-11-29-raspbian-stretch-lite.img").unwrap();
    let mut shared_vfat = ::vfat::VFat::from(file).unwrap();
    let mut config_txt = shared_vfat
        .open("/config.txt")
        .unwrap()
        .into_file()
        .unwrap();
    let mut contents = String::new();
    config_txt.read_to_string(&mut contents).unwrap();
    config_txt.seek(SeekFrom::Start(0)).unwrap();
    for i in 1..contents.len() - 2 {
        let mut contents2 = String::new();
        config_txt.seek(SeekFrom::Start(0)).unwrap();
        config_txt.seek(SeekFrom::Current(i as i64)).unwrap();
        config_txt.read_to_string(&mut contents2).unwrap();
        println!("{:?}", contents2);
        assert_eq!(&contents[i..], &contents2[..]);
    }
}
