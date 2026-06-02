// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! 零成本雜湊器枚舉 — 以 static dispatch 取代 `Box<dyn DynHasher>`。
//!
//! # 設計
//!
//! - **零 heap alloc**: `HasherEnum` 是 stack-allocated flat enum，14 個變體共用同一塊記憶體
//! - **Static dispatch**: `update()` / `finalize()` 透過 `match` 分發到具體類型，編譯器可完全 inline
//! - **Clone**: 每個變體內層類型均實作 `Clone`（`RustCrypto` 的 `CoreWrapper<T>`），複製約 64–200 bytes
//! - **Send + Sync**: 所有內層 crate 類型均為 `Send + Sync`

use super::{
    Blake2b512Hasher, Blake2s256Hasher, Blake3Hasher, Md5Hasher, Sha1Hasher, Sha224Hasher,
    Sha256Hasher, Sha384Hasher, Sha3_256Hasher, Sha3_512Hasher, Sha512Hasher, Shake128XofHasher,
    Shake256XofHasher,
};

/// 14 種雜湊演算法的零成本靜態分發枚舉
///
/// # 使用範例
///
/// ```rust
/// use onee_checker::hasher::{HasherEnum, Sha256Hasher};
///
/// let mut h = HasherEnum::Sha256(Sha256Hasher::new());
/// h.update(b"hello world");
/// let hash = h.finalize();
/// assert_eq!(hash.len(), 32);
/// ```
///
/// # 設計取捨
///
/// BLAKE3 內部狀態較大（~1928 bytes）導致 `HasherEnum` 整體大小約 1936 bytes。
/// 不 boxing 的原因是保持 stack allocation 與零成本 dispatch。
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum HasherEnum {
    Md5(Md5Hasher),
    Sha1(Sha1Hasher),
    Sha224(Sha224Hasher),
    Sha256(Sha256Hasher),
    Sha384(Sha384Hasher),
    Sha512(Sha512Hasher),
    Sha3_256(Sha3_256Hasher),
    Sha3_512(Sha3_512Hasher),
    Shake128(Shake128XofHasher),
    Shake256(Shake256XofHasher),
    Blake2s256(Blake2s256Hasher),
    Blake2b512(Blake2b512Hasher),
    Blake3(Blake3Hasher),
}

impl HasherEnum {
    /// 將資料餵入雜湊器
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        match self {
            Self::Md5(h) => h.update(data),
            Self::Sha1(h) => h.update(data),
            Self::Sha224(h) => h.update(data),
            Self::Sha256(h) => h.update(data),
            Self::Sha384(h) => h.update(data),
            Self::Sha512(h) => h.update(data),
            Self::Sha3_256(h) => h.update(data),
            Self::Sha3_512(h) => h.update(data),
            Self::Shake128(h) => h.update(data),
            Self::Shake256(h) => h.update(data),
            Self::Blake2s256(h) => h.update(data),
            Self::Blake2b512(h) => h.update(data),
            Self::Blake3(h) => h.update(data),
        }
    }

    /// 一次將整個 buffer 餵入雜湊器。
    ///
    /// 與 `update()` 功能相同，但語意上表示呼叫者已準備好全部資料。
    /// BLAKE3 多線程優化在更高層的 `blake3_hash_bulk()` 中實作。
    #[inline]
    pub fn update_bulk(&mut self, data: &[u8]) {
        // BLAKE3 多線程由 higher-level `blake3_hash_bulk()` 函式處理；
        // 此處統一使用串流 update 以保持 enum dispatch 的一致性。
        self.update(data);
    }

    /// 完成計算並回傳雜湊值（consumes self）
    #[inline]
    #[must_use]
    pub fn finalize(self) -> Vec<u8> {
        match self {
            Self::Md5(h) => h.finish(),
            Self::Sha1(h) => h.finish(),
            Self::Sha224(h) => h.finish(),
            Self::Sha256(h) => h.finish(),
            Self::Sha384(h) => h.finish(),
            Self::Sha512(h) => h.finish(),
            Self::Sha3_256(h) => h.finish(),
            Self::Sha3_512(h) => h.finish(),
            Self::Shake128(h) => h.finish(),
            Self::Shake256(h) => h.finish(),
            Self::Blake2s256(h) => h.finish(),
            Self::Blake2b512(h) => h.finish(),
            Self::Blake3(h) => h.finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_clone_produces_independent_hasher() {
        let h1 = HasherEnum::Sha256(Sha256Hasher::new());
        let mut h2 = h1.clone();

        let _ = h1.finalize();
        h2.update(b"hello");
        let result = h2.finalize();
        assert_eq!(
            hex::encode(&result),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_all_algorithms_output_correct_length() {
        let data = b"quick brown fox";

        let cases: Vec<(HasherEnum, usize)> = vec![
            (HasherEnum::Md5(Md5Hasher::new()), 16),
            (HasherEnum::Sha1(Sha1Hasher::new()), 20),
            (HasherEnum::Sha256(Sha256Hasher::new()), 32),
            (HasherEnum::Sha512(Sha512Hasher::new()), 64),
            (HasherEnum::Sha3_256(Sha3_256Hasher::new()), 32),
            (HasherEnum::Shake128(Shake128XofHasher::new(32)), 32),
            (HasherEnum::Blake2b512(Blake2b512Hasher::new()), 64),
            (HasherEnum::Blake3(Blake3Hasher::new(32)), 32),
        ];

        for (mut h, expected_len) in cases {
            h.update(data);
            let result = h.finalize();
            assert_eq!(result.len(), expected_len);
        }
    }

    #[test]
    fn test_clone_reuse_same_result() {
        let mut h1 = HasherEnum::Sha256(Sha256Hasher::new());
        let mut h2 = h1.clone();

        h1.update(b"same data");
        h2.update(b"same data");

        assert_eq!(h1.finalize(), h2.finalize());
    }
}
