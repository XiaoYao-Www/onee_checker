// blake3 哈希器
use super::DynHasher;
use blake3::{Hasher, OutputReader};


/// ### BLAKE3 哈希器結構
/// 
/// - inner 內部 blake3 雜湊器
/// - out_len 輸出長度（位元組）
pub struct Blake3Hasher {
    inner: blake3::Hasher,
    out_len: usize,
}

impl DynHasher for Blake3Hasher {
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    fn finalize(self: Box<Self>) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0u8; self.out_len];
        let mut reader: OutputReader = self.inner.finalize_xof();
        reader.fill(&mut buf);
        buf
    }
}

impl Blake3Hasher {
    pub fn new(out_len: usize) -> Self {
        Self {
            inner: Hasher::new(),
            out_len,
        }
    }
}
