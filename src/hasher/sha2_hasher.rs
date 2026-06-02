// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! SHA-2 哈希器家族 (SHA-224, SHA-256, SHA-384, SHA-512)

use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

macro_rules! sha2_hasher {
    ($name:ident, $inner:ty) => {
        #[derive(Clone)]
        pub struct $name($inner);

        impl $name {
            pub fn new() -> Self {
                Self(<$inner>::new())
            }

            pub fn update(&mut self, data: &[u8]) {
                self.0.update(data);
            }

            pub fn finish(self) -> Vec<u8> {
                self.0.finalize().to_vec()
            }
        }
    };
}

sha2_hasher!(Sha224Hasher, Sha224);
sha2_hasher!(Sha256Hasher, Sha256);
sha2_hasher!(Sha384Hasher, Sha384);
sha2_hasher!(Sha512Hasher, Sha512);