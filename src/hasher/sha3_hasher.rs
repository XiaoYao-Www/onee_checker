// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! SHA-3 哈希器家族 (SHA3-256, SHA3-512, SHAKE128, SHAKE256)

use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Digest, Sha3_256, Sha3_512, Shake128, Shake256,
};

// ── 固定長度 SHA-3 ──

#[derive(Clone)]
pub struct Sha3_256Hasher(Sha3_256);

impl Sha3_256Hasher {
    pub fn new() -> Self {
        Self(Sha3_256::new())
    }

    pub fn update(&mut self, data: &[u8]) {
        Digest::update(&mut self.0, data);
    }

    pub fn finish(self) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

#[derive(Clone)]
pub struct Sha3_512Hasher(Sha3_512);

impl Sha3_512Hasher {
    pub fn new() -> Self {
        Self(Sha3_512::new())
    }

    pub fn update(&mut self, data: &[u8]) {
        Digest::update(&mut self.0, data);
    }

    pub fn finish(self) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}

// ── 可變長度 SHAKE ──

#[derive(Clone)]
pub struct Shake128XofHasher {
    inner: Shake128,
    out_len: u16,
}

impl Shake128XofHasher {
    pub fn new(out_len: u16) -> Self {
        Self {
            inner: Shake128::default(),
            out_len,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        Update::update(&mut self.inner, data);
    }

    pub fn finish(self) -> Vec<u8> {
        let mut reader = self.inner.finalize_xof();
        let mut buf = vec![0u8; self.out_len as usize];
        reader.read(&mut buf);
        buf
    }
}

#[derive(Clone)]
pub struct Shake256XofHasher {
    inner: Shake256,
    out_len: u16,
}

impl Shake256XofHasher {
    pub fn new(out_len: u16) -> Self {
        Self {
            inner: Shake256::default(),
            out_len,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        Update::update(&mut self.inner, data);
    }

    pub fn finish(self) -> Vec<u8> {
        let mut reader = self.inner.finalize_xof();
        let mut buf = vec![0u8; self.out_len as usize];
        reader.read(&mut buf);
        buf
    }
}