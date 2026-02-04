// md5 哈希器
use super::DynHasher;
use md5::{Digest, Md5};


/// ### MD5 哈希器結構
pub struct Md5Hasher(Md5);

impl DynHasher for Md5Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl Md5Hasher {
    pub fn new() -> Self {
        Self(Md5::new())
    }
}