// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! BLAKE3 哈希器（可變輸出長度）

#[derive(Clone)]
pub struct Blake3Hasher {
    inner: blake3::Hasher,
    out_len: u16,
}

impl Blake3Hasher {
    pub fn new(out_len: u16) -> Self {
        Self {
            inner: blake3::Hasher::new(),
            out_len,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    pub fn finish(self) -> Vec<u8> {
        let mut buf = vec![0u8; self.out_len as usize];
        self.inner.finalize_xof().fill(&mut buf);
        buf
    }
}