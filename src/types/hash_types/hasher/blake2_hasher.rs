// blake2 哈希器
use super::DynHasher;
use blake2::{Digest, Blake2b512, Blake2s256};


/// ### BLAKE2b-512 哈希器結構
pub struct Blake2b512Hasher(Blake2b512);

impl DynHasher for Blake2b512Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl Blake2b512Hasher {
    pub fn new() -> Self {
        Self(Blake2b512::new())
    }
}

/// ### BLAKE2s-256 哈希器結構
pub struct Blake2s256Hasher(Blake2s256);

impl DynHasher for Blake2s256Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl Blake2s256Hasher {
    pub fn new() -> Self {
        Self(Blake2s256::new())
    }
}