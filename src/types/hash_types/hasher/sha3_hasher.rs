// sha3 哈希器
use super::DynHasher;
use sha3::{digest::{Update, ExtendableOutput, XofReader}, Digest, Sha3_256, Sha3_512, Shake128, Shake256};

/// ### SHA-3-256 哈希器結構
pub struct Sha3_256Hasher(Sha3_256);

impl DynHasher for Sha3_256Hasher {
    fn update(&mut self, data: &[u8]) {
        Digest::update(&mut self.0, data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl Sha3_256Hasher {
    pub fn new() -> Self {
        Self(Sha3_256::new())
    }
}

/// ### SHA-3-512 哈希器結構
pub struct Sha3_512Hasher(Sha3_512);

impl DynHasher for Sha3_512Hasher {
    fn update(&mut self, data: &[u8]) {
        Digest::update(&mut self.0, data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

impl Sha3_512Hasher {
    pub fn new() -> Self {
        Self(Sha3_512::new())
    }
}

/// ### SHAKE-128 哈希器結構
/// 
/// - inner 內部 SHAKE-128 雜湊器
/// - out_len 輸出長度（位元組）
pub struct Shake128XofHasher {
    inner: Shake128,
    out_len: usize,
}

impl DynHasher for Shake128XofHasher {
    fn update(&mut self, data: &[u8]) {
        Update::update(&mut self.inner, data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        let mut reader = self.inner.finalize_xof();
        let mut buf: Vec<u8> = vec![0u8; self.out_len];
        reader.read(&mut buf);
        buf
    }
}

impl Shake128XofHasher {
    pub fn new(out_len: usize) -> Self {
        Self {
            inner: Shake128::default(),
            out_len,
        }
    }
}

/// ### SHAKE-256 哈希器結構
/// 
/// - inner 內部 SHAKE-256 雜湊器
/// - out_len 輸出長度（位元組）
pub struct Shake256XofHasher {
    inner: Shake256,
    out_len: usize,
}

impl DynHasher for Shake256XofHasher {
    fn update(&mut self, data: &[u8]) {
        Update::update(&mut self.inner, data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        let mut reader = self.inner.finalize_xof();
        let mut buf: Vec<u8> = vec![0u8; self.out_len];
        reader.read(&mut buf);
        buf
    }
}

impl Shake256XofHasher {
    pub fn new(out_len: usize) -> Self {
        Self {
            inner: Shake256::default(),
            out_len,
        }
    }
}