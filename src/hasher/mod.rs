// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! 哈希器實作 — 每個演算法獨立檔案，加上 `HasherEnum` 零成本靜態分發。

mod enum_hasher;
mod md5_hasher;
mod sha1_hasher;
mod sha2_hasher;
mod sha3_hasher;
mod blake2_hasher;
mod blake3_hasher;

pub use enum_hasher::*;
pub use md5_hasher::*;
pub use sha1_hasher::*;
pub use sha2_hasher::*;
pub use sha3_hasher::*;
pub use blake2_hasher::*;
pub use blake3_hasher::*;
