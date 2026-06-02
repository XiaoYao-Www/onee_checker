// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! BLAKE2 哈希器家族 (BLAKE2s-256, BLAKE2b-512)

use blake2::{Blake2b512, Blake2s256, Digest};

macro_rules! blake2_hasher {
    ($name:ident, $inner:ty) => {
        #[derive(Clone)]
        pub struct $name($inner);

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(<$inner>::new())
            }

            pub fn update(&mut self, data: &[u8]) {
                self.0.update(data);
            }

            #[must_use]
            pub fn finish(self) -> Vec<u8> {
                self.0.finalize().to_vec()
            }
        }
    };
}

blake2_hasher!(Blake2b512Hasher, Blake2b512);
blake2_hasher!(Blake2s256Hasher, Blake2s256);
