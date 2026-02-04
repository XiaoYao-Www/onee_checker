// sha1 哈希器
use super::DynHasher;
use sha1::{Digest, Sha1};


/// ### SHA-1 哈希器結構
pub struct Sha1Hasher(Sha1);

impl DynHasher for Sha1Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.0.finalize().to_vec()
    } 
}

impl Sha1Hasher {
    pub fn new() -> Self {
        Self(Sha1::new())
    }
}