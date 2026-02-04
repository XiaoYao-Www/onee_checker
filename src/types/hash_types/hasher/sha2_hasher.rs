// sha2 哈希器
use super::DynHasher;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};


/// ### SHA-224 哈希器結構
pub struct Sha224Hasher(Sha224);

impl DynHasher for Sha224Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl Sha224Hasher {
    pub fn new() -> Self {
        Self(Sha224::new())
    }
}

/// ### SHA-256 哈希器結構
pub struct Sha256Hasher(Sha256);

impl DynHasher for Sha256Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl Sha256Hasher {
    pub fn new() -> Self {
        Self(Sha256::new())
    }
}

/// ### SHA-384 哈希器結構
pub struct Sha384Hasher(Sha384);

impl DynHasher for Sha384Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl Sha384Hasher {
    pub fn new() -> Self {
        Self(Sha384::new())
    }
}

/// ### SHA-512 哈希器結構
pub struct Sha512Hasher(Sha512);

impl DynHasher for Sha512Hasher {
    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl Sha512Hasher {
    pub fn new() -> Self {
        Self(Sha512::new())
    }
}