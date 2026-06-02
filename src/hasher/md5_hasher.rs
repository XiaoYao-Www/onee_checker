// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! MD5 哈希器

use md5::{Digest, Md5};

#[derive(Clone)]
pub struct Md5Hasher(Md5);

impl Default for Md5Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Md5Hasher {
    #[must_use]
    pub fn new() -> Self {
        Self(Md5::new())
    }

    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}
