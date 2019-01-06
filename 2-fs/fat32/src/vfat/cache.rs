use std::{fmt, io};
use std::io::Read;
use std::collections::HashMap;

use traits::BlockDevice;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool,
}

pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64,
}

pub struct CachedDevice {
    device: Box<BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition,
}

impl CachedDevice {
    /// Creates a new `CachedDevice` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `n` _before_
    /// `partition.start` is made to physical sector `n`. Cached sectors before
    /// `partition.start` are the size of a physical sector. An access to a
    /// sector `n` at or after `partition.start` is made to the _logical_ sector
    /// `n - partition.start`. Cached sectors at or after `partition.start` are
    /// the size of a logical sector, `partition.sector_size`.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> CachedDevice
    where
        T: BlockDevice + 'static,
    {
        assert!(partition.sector_size >= device.sector_size());

        CachedDevice {
            device: Box::new(device),
            cache: HashMap::new(),
            partition: partition,
        }
    }

    /// Maps a user's request for a sector `virt` to the physical sector and
    /// number of physical sectors required to access `virt`.
    fn virtual_to_physical(&self, virt: u64) -> (u64, u64) {
        if self.device.sector_size() == self.partition.sector_size {
            (virt, 1)
        } else if virt < self.partition.start {
            (virt, 1)
        } else {
            let factor = self.partition.sector_size / self.device.sector_size();
            let logical_offset = virt - self.partition.start;
            let physical_offset = logical_offset * factor;
            let physical_sector = self.partition.start + physical_offset;
            (physical_sector, factor)
        }
    }

    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        self.get(sector)?;
        let entry = self.cache.get_mut(&sector).unwrap();
        entry.dirty = true;
        Ok(entry.data.as_mut_slice())
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        if !self.cache.contains_key(&sector) {
            let (physical_sector, physical_sector_num) = self.virtual_to_physical(sector);
            let physical_sector_size = self.device.sector_size();
            let logical_sector_size = (physical_sector_size * physical_sector_num) as usize;
            let mut new_vec = Vec::with_capacity(logical_sector_size);
            new_vec.resize(logical_sector_size, 0);
            for i in 0..physical_sector_num {
                self.device.read_sector(
                    physical_sector + i,
                    &mut new_vec[(i * physical_sector_size) as usize..],
                )?;
            }
            self.cache.insert(sector, CacheEntry {
                data: new_vec,
                dirty: false
            });
        }
        Ok(&self.cache[&sector].data[..])
    }
}

// FIXME: Implement `BlockDevice` for `CacheDevice`. The `read_sector` and
// `write_sector` methods should only read/write from/to cached sectors.
impl BlockDevice for CachedDevice {
    fn sector_size(&self) -> u64 {
        self.partition.sector_size
    }

    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        if self.cache.contains_key(&n) {
            let cached_entry = &self.cache[&n];
            io::Cursor::new(&cached_entry.data).read(buf)
        } else {
            Err(io::Error::new(io::ErrorKind::Interrupted, "data not cached yet"))
        }
    }

    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
        unimplemented!();
    }
}

impl fmt::Debug for CachedDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut keys: Vec<&u64> = self.cache.keys().collect();
        keys.sort();
        f.debug_struct("CachedDevice")
            .field("device", &"<block device>")
            .field("cached sectors", &keys)
            .finish()
    }
}
