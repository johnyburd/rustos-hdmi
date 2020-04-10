#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct Cluster(u32);

impl From<u32> for Cluster {
    fn from(raw_num: u32) -> Cluster {
        Cluster(raw_num & !(0xF << 28))
    }
}

impl Cluster {
    pub fn get_offset(&self) -> u64 {
        self.0.saturating_sub(2) as u64
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}
