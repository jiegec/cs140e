use vfat::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct Cluster(u32);

impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }
}

// TODO: Implement any useful helper methods on `Cluster`.
impl Cluster {
    pub(super) fn cluster_num(&self) -> u32 {
        self.0
    }

    pub(super) fn cluster_index(&self) -> u32 {
        // Cluster start from 2
        self.0 - 2
    }

    pub(super) fn is_valid(&self) -> bool {
        self.0 >= 2
    }
}
