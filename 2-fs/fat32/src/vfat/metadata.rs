use std::fmt;

use traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub struct Timestamp {
    pub(super) date: Date,
    pub(super) time: Time,
}

/// Metadata for a directory entry.
#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub struct Metadata {
    // FIXME: Fill me in.
    pub(super) attributes: Attributes,
    __r1: u8,
    creation_time_tenths_seconds: u8,
    creation_time: Time,
    creation_date: Date,
    access_date: Date,
    high_two_bytes_first_cluster: u16,
    last_modified_time: Time,
    last_modified_date: Date,
    low_two_bytes_first_cluster: u16,
}

// FIXME: Implement `traits::Timestamp` for `Timestamp`.
impl traits::Timestamp for Timestamp {
    fn year(&self) -> usize {
        ((self.date.0 >> 9) & 0x7F) as usize + 1980
    }

    /// The calendar month, starting at 1 for January. Always in range [1, 12].
    ///
    /// January is 1, Feburary is 2, ..., December is 12.
    fn month(&self) -> u8 {
        ((self.date.0 >> 5) & 0xF) as u8
    }

    /// The calendar day, starting at 1. Always in range [1, 31].
    fn day(&self) -> u8 {
        (self.date.0 & 0x1F) as u8
    }

    /// The 24-hour hour. Always in range [0, 24).
    fn hour(&self) -> u8 {
        ((self.time.0 >> 11) & 0x1F) as u8
    }

    /// The minute. Always in range [0, 60).
    fn minute(&self) -> u8 {
        ((self.time.0 >> 5) & 0x3F) as u8
    }

    /// The second. Always in range [0, 60).
    fn second(&self) -> u8 {
        (self.time.0 & 0x1F) as u8 * 2
    }
}

impl Attributes {
    fn read_only(&self) -> bool {
        (self.0 & 0x01) != 0
    }

    fn hidden(&self) -> bool {
        (self.0 & 0x02) != 0
    }

    fn system(&self) -> bool {
        (self.0 & 0x04) != 0
    }

    fn volume_id(&self) -> bool {
        (self.0 & 0x08) != 0
    }

    pub(super) fn directory(&self) -> bool {
        (self.0 & 0x10) != 0
    }

    fn archive(&self) -> bool {
        (self.0 & 0x20) != 0
    }

    pub(super) fn lfn(&self) -> bool {
        self.0 == 0x0F
    }
}

// FIXME: Implement `traits::Metadata` for `Metadata`.
impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    fn read_only(&self) -> bool {
        self.attributes.read_only()
    }

    fn hidden(&self) -> bool {
        self.attributes.hidden()
    }

    fn system(&self) -> bool {
        self.attributes.system()
    }

    fn volume_id(&self) -> bool {
        self.attributes.volume_id()
    }

    fn archive(&self) -> bool {
        self.attributes.archive()
    }

    fn created(&self) -> Self::Timestamp {
        Timestamp {
            // TODO: deal with tenth second?
            time: self.creation_time,
            date: self.creation_date,
        }
    }

    fn accessed(&self) -> Self::Timestamp {
        Timestamp {
            time: Time(0),
            date: self.access_date,
        }
    }

    fn modified(&self) -> Self::Timestamp {
        Timestamp {
            time: self.last_modified_time,
            date: self.last_modified_date,
        }
    }
}

// FIXME: Implement `fmt::Display` (to your liking) for `Metadata`.
impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use traits::Metadata;
        f.debug_struct("Metadata")
            .field("read_only", &self.read_only())
            .field("hidden", &self.hidden())
            .field("created", &self.created())
            .field("accessed", &self.accessed())
            .field("modified", &self.modified())
            .finish()
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use traits::Timestamp;
        f.write_fmt(format_args!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year(),
            self.month(),
            self.day(),
            self.hour(),
            self.minute(),
            self.second()
        ))
    }
}

impl Metadata {
    pub(super) fn first_cluster(&self) -> u32 {
        ((self.high_two_bytes_first_cluster as u32) << 16) | self.low_two_bytes_first_cluster as u32
    }
}