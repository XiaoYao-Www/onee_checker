// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! SHA-1 哈希器

use sha1::{Digest, Sha1};

#[derive(Clone)]
pub struct Sha1Hasher(Sha1);

impl Default for Sha1Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha1Hasher {
    #[must_use]
    pub fn new() -> Self {
        Self(Sha1::new())
    }

    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.0.finalize().to_vec()
    }
}
