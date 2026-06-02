// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! 核心演算法類型 — `HashType` 枚舉、`HashData` 結構、`HashSpec` 演算法描述。

use std::path::PathBuf;

use crate::error::{OneeError, Result};
use crate::hasher::{
    Blake2b512Hasher, Blake2s256Hasher, Blake3Hasher, HasherEnum, Md5Hasher, Sha1Hasher,
    Sha224Hasher, Sha256Hasher, Sha384Hasher, Sha3_256Hasher, Sha3_512Hasher, Sha512Hasher,
    Shake128XofHasher, Shake256XofHasher,
};

// ──────────────────────────────────────────────────────────
//  HashType 枚舉
// ──────────────────────────────────────────────────────────

/// 所有支援的 hash 演算法。
///
/// 有固定長度的變體不帶參數；可變長度的（SHAKE、BLAKE3）攜帶 `u16` 輸出長度（bytes）。
#[derive(Debug, Clone, PartialEq)]
pub enum HashType {
    // ── MD ──
    MD5,
    // ── SHA-1 ──
    SHA1,
    // ── SHA-2 ──
    SHA224,
    SHA256,
    SHA384,
    SHA512,
    // ── SHA-3 ──
    SHA3_256,
    SHA3_512,
    SHAKE128(u16),
    SHAKE256(u16),
    // ── BLAKE2 ──
    BLAKE2S256,
    BLAKE2B512,
    // ── BLAKE3 ──
    BLAKE3(u16),
}

impl HashType {
    /// 建立對應的 hasher 實例（零 heap alloc 的 enum dispatch）
    #[must_use]
    pub fn create_hasher(&self) -> HasherEnum {
        match self {
            Self::MD5 => HasherEnum::Md5(Md5Hasher::new()),
            Self::SHA1 => HasherEnum::Sha1(Sha1Hasher::new()),
            Self::SHA224 => HasherEnum::Sha224(Sha224Hasher::new()),
            Self::SHA256 => HasherEnum::Sha256(Sha256Hasher::new()),
            Self::SHA384 => HasherEnum::Sha384(Sha384Hasher::new()),
            Self::SHA512 => HasherEnum::Sha512(Sha512Hasher::new()),
            Self::SHA3_256 => HasherEnum::Sha3_256(Sha3_256Hasher::new()),
            Self::SHA3_512 => HasherEnum::Sha3_512(Sha3_512Hasher::new()),
            Self::SHAKE128(len) => HasherEnum::Shake128(Shake128XofHasher::new(*len)),
            Self::SHAKE256(len) => HasherEnum::Shake256(Shake256XofHasher::new(*len)),
            Self::BLAKE2S256 => HasherEnum::Blake2s256(Blake2s256Hasher::new()),
            Self::BLAKE2B512 => HasherEnum::Blake2b512(Blake2b512Hasher::new()),
            Self::BLAKE3(len) => HasherEnum::Blake3(Blake3Hasher::new(*len)),
        }
    }

    /// 該演算法是否支援自訂輸出長度
    #[must_use]
    pub const fn can_specify_length(&self) -> bool {
        matches!(self, Self::SHAKE128(_) | Self::SHAKE256(_) | Self::BLAKE3(_))
    }

    /// 預設輸出長度（bytes）。固定長度演算法回傳其固定值。
    #[must_use]
    pub const fn default_length(&self) -> u16 {
        match self {
            Self::MD5 => 16,
            Self::SHA1 => 20,
            Self::SHA224 => 28,
            Self::SHA256
            | Self::SHA3_256
            | Self::SHAKE128(_)
            | Self::BLAKE2S256
            | Self::BLAKE3(_) => 32,
            Self::SHA384 => 48,
            Self::SHA512 | Self::SHA3_512 | Self::SHAKE256(_) | Self::BLAKE2B512 => 64,
        }
    }

    /// 雜湊值的位元長度（`default_length() * 8`）
    #[must_use]
    pub const fn bit_length(&self) -> usize {
        (self.default_length() as usize) * 8
    }

    /// 檔案副檔名（不含點），例如 `"sha256"`、`"shake128-256"`、`"blake3-256"`
    #[must_use]
    pub fn suffix(&self) -> String {
        match self {
            Self::MD5 => "md5".into(),
            Self::SHA1 => "sha1".into(),
            Self::SHA224 => "sha224".into(),
            Self::SHA256 => "sha256".into(),
            Self::SHA384 => "sha384".into(),
            Self::SHA512 => "sha512".into(),
            Self::SHA3_256 => "sha3_256".into(),
            Self::SHA3_512 => "sha3_512".into(),
            Self::SHAKE128(len) => format!("shake128-{}", len * 8),
            Self::SHAKE256(len) => format!("shake256-{}", len * 8),
            Self::BLAKE2S256 => "blake2s256".into(),
            Self::BLAKE2B512 => "blake2b512".into(),
            Self::BLAKE3(len) => format!("blake3-{}", len * 8),
        }
    }

    /// 人類可讀的算法名稱（用於錯誤訊息）
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MD5 => "MD5",
            Self::SHA1 => "SHA-1",
            Self::SHA224 => "SHA-224",
            Self::SHA256 => "SHA-256",
            Self::SHA384 => "SHA-384",
            Self::SHA512 => "SHA-512",
            Self::SHA3_256 => "SHA3-256",
            Self::SHA3_512 => "SHA3-512",
            Self::SHAKE128(_) => "SHAKE-128",
            Self::SHAKE256(_) => "SHAKE-256",
            Self::BLAKE2S256 => "BLAKE2s-256",
            Self::BLAKE2B512 => "BLAKE2b-512",
            Self::BLAKE3(_) => "BLAKE3",
        }
    }
}

// ──────────────────────────────────────────────────────────
//  HashData — 單一檔案計算結果
// ──────────────────────────────────────────────────────────

/// 單一檔案的 hash 計算結果。
#[derive(Debug, Clone)]
pub struct HashData {
    /// 檔案路徑
    pub path: PathBuf,
    /// 原始 hash bytes
    pub hash_bytes: Vec<u8>,
    /// 使用的演算法
    pub hash_type: HashType,
}

impl HashData {
    /// 建立新的 `HashData`
    #[must_use]
    pub fn new(path: PathBuf, hash_bytes: Vec<u8>, hash_type: HashType) -> Self {
        Self { path, hash_bytes, hash_type }
    }

    /// 回傳十六進位 hash 字串（分配新 String）
    #[must_use]
    pub fn hash_hex(&self) -> String {
        let mut buf = String::with_capacity(self.hash_bytes.len() * 2);
        self.hash_hex_into(&mut buf);
        buf
    }

    /// 將十六進位 hash 寫入外部緩衝區（零 alloc）
    ///
    /// ```rust
    /// use onee_checker::algorithm::{HashData, HashType};
    /// let data = HashData::new("/x".into(), vec![0xab, 0xcd], HashType::SHA256);
    /// let mut buf = String::new();
    /// data.hash_hex_into(&mut buf);
    /// assert_eq!(buf, "abcd");
    /// ```
    pub fn hash_hex_into(&self, buf: &mut String) {
        buf.reserve(self.hash_bytes.len() * 2);
        buf.push_str(&hex::encode(&self.hash_bytes));
    }
}

// ──────────────────────────────────────────────────────────
//  HashAlgo — CLI 參數用枚舉
// ──────────────────────────────────────────────────────────

/// CLI 參數用的 hash 演算法枚舉（不攜帶長度資訊）。
///
/// 對應到 `HashType`，但分離了 CLI 與內部類型的關注點。
#[derive(Debug, Clone, clap::ValueEnum, PartialEq)]
pub enum HashAlgo {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    #[clap(name = "sha3-256")]
    Sha3256,
    #[clap(name = "sha3-512")]
    Sha3512,
    #[clap(name = "shake128")]
    Shake128,
    #[clap(name = "shake256")]
    Shake256,
    #[clap(name = "blake2s256")]
    Blake2s256,
    #[clap(name = "blake2b512")]
    Blake2b512,
    #[clap(name = "blake3")]
    Blake3,
}

impl HashAlgo {
    /// 將 CLI 枚舉轉換為內部 `HashType`，加上可選的長度參數。
    ///
    /// `length` 為 `None` 時使用預設長度。
    pub fn to_hash_type(&self, length: Option<u16>) -> Result<HashType> {
        let len = length.unwrap_or_else(|| self.default_length());
        match self {
            Self::Md5 => Ok(HashType::MD5),
            Self::Sha1 => Ok(HashType::SHA1),
            Self::Sha224 => Ok(HashType::SHA224),
            Self::Sha256 => Ok(HashType::SHA256),
            Self::Sha384 => Ok(HashType::SHA384),
            Self::Sha512 => Ok(HashType::SHA512),
            Self::Sha3256 => Ok(HashType::SHA3_256),
            Self::Sha3512 => Ok(HashType::SHA3_512),
            Self::Shake128 => Ok(HashType::SHAKE128(len)),
            Self::Shake256 => Ok(HashType::SHAKE256(len)),
            Self::Blake2s256 => Ok(HashType::BLAKE2S256),
            Self::Blake2b512 => Ok(HashType::BLAKE2B512),
            Self::Blake3 => Ok(HashType::BLAKE3(len)),
        }
    }

    /// 該 CLI 算法是否支援自訂長度
    #[must_use]
    pub const fn can_specify_length(&self) -> bool {
        matches!(self, Self::Shake128 | Self::Shake256 | Self::Blake3)
    }

    /// 預設長度（bytes）
    #[must_use]
    pub const fn default_length(&self) -> u16 {
        match self {
            Self::Md5 => 16,
            Self::Sha1 => 20,
            Self::Sha224 => 28,
            Self::Sha256 | Self::Sha3256 | Self::Shake128 | Self::Blake2s256 | Self::Blake3 => 32,
            Self::Sha384 => 48,
            Self::Sha512 | Self::Sha3512 | Self::Shake256 | Self::Blake2b512 => 64,
        }
    }

    /// 從副檔名猜測演算法
    #[must_use]
    pub fn from_suffix(suffix: &str) -> Option<Self> {
        match suffix {
            "md5" => Some(Self::Md5),
            "sha1" => Some(Self::Sha1),
            "sha224" => Some(Self::Sha224),
            "sha256" => Some(Self::Sha256),
            "sha384" => Some(Self::Sha384),
            "sha512" => Some(Self::Sha512),
            "sha3_256" => Some(Self::Sha3256),
            "sha3_512" => Some(Self::Sha3512),
            s if s.starts_with("shake128") => Some(Self::Shake128),
            s if s.starts_with("shake256") => Some(Self::Shake256),
            "blake2s256" => Some(Self::Blake2s256),
            "blake2b512" => Some(Self::Blake2b512),
            s if s.starts_with("blake3") => Some(Self::Blake3),
            _ => None,
        }
    }
}

// ──────────────────────────────────────────────────────────
//  BufferSize — human-readable buffer 解析
// ──────────────────────────────────────────────────────────

/// 預設緩衝區大小（1 MiB）
pub const DEFAULT_BUFFER_SIZE: usize = 1024 * 1024;

/// 最小緩衝區大小（512 bytes）
const MIN_BUFFER_SIZE: usize = 512;

/// 人類可讀的 buffer 大小，例如 `4K`、`1M`、`64K`。
///
/// 支援後綴：`B` (bytes)、`K` (KiB)、`M` (MiB)、`G` (GiB)。
/// 無後綴時視為 bytes。
#[derive(Debug, Clone)]
pub struct BufferSize(pub usize);

impl BufferSize {
    /// 預設緩衝區大小（1 MiB）
    pub const DEFAULT: Self = Self(DEFAULT_BUFFER_SIZE);
}

impl std::str::FromStr for BufferSize {
    type Err = OneeError;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim().to_lowercase();

        if s.is_empty() {
            return Err(OneeError::ArgumentError("buffer size 不可為空".into()));
        }

        let (num_part, unit) = if s.ends_with('b') {
            let (n, _) = s.split_at(s.len() - 1);
            (n, 'b')
        } else if s.ends_with('k') {
            let (n, _) = s.split_at(s.len() - 1);
            (n, 'k')
        } else if s.ends_with('m') {
            let (n, _) = s.split_at(s.len() - 1);
            (n, 'm')
        } else if s.ends_with('g') {
            let (n, _) = s.split_at(s.len() - 1);
            (n, 'g')
        } else {
            (&s[..], 'b')
        };

        let num: usize = num_part
            .parse()
            .map_err(|_| OneeError::ArgumentError(format!("無法解析 buffer size: {s}")))?;

        let bytes = match unit {
            'b' => num,
            'k' => num * 1024,
            'm' => num * 1024 * 1024,
            'g' => num * 1024 * 1024 * 1024,
            _ => unreachable!(),
        };

        if bytes < MIN_BUFFER_SIZE {
            return Err(OneeError::ArgumentError(format!(
                "buffer size 不可小於 {MIN_BUFFER_SIZE} bytes: 收到 {bytes} bytes"
            )));
        }

        Ok(Self(bytes))
    }
}

impl std::fmt::Display for BufferSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0;
        if bytes >= 1024 * 1024 * 1024 {
            write!(f, "{}G", bytes / (1024 * 1024 * 1024))
        } else if bytes >= 1024 * 1024 {
            write!(f, "{}M", bytes / (1024 * 1024))
        } else if bytes >= 1024 {
            write!(f, "{}K", bytes / 1024)
        } else {
            write!(f, "{bytes}B")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_size_parsing() {
        assert_eq!("4K".parse::<BufferSize>().unwrap().0, 4096);
        assert_eq!("1M".parse::<BufferSize>().unwrap().0, 1024 * 1024);
        assert_eq!("64K".parse::<BufferSize>().unwrap().0, 65536);
        assert_eq!("512".parse::<BufferSize>().unwrap().0, 512);
        assert_eq!("2G".parse::<BufferSize>().unwrap().0, 2 * 1024 * 1024 * 1024);
        assert!("0K".parse::<BufferSize>().is_err());
        assert!("".parse::<BufferSize>().is_err());
        assert!("abc".parse::<BufferSize>().is_err());
    }

    #[test]
    fn test_buffer_size_too_small() {
        assert!("1".parse::<BufferSize>().is_err()); // 1 byte < 512
        assert!("511B".parse::<BufferSize>().is_err());
        assert!("512B".parse::<BufferSize>().is_ok());
        assert_eq!(BufferSize::DEFAULT.0, DEFAULT_BUFFER_SIZE);
    }

    #[test]
    fn test_hash_type_all_variants_create_hasher() {
        // 確認 14+ 種變體均可 create_hasher
        let variants = [
            HashType::MD5,
            HashType::SHA1,
            HashType::SHA224,
            HashType::SHA256,
            HashType::SHA384,
            HashType::SHA512,
            HashType::SHA3_256,
            HashType::SHA3_512,
            HashType::SHAKE128(32),
            HashType::SHAKE256(64),
            HashType::BLAKE2S256,
            HashType::BLAKE2B512,
            HashType::BLAKE3(32),
            HashType::BLAKE3(64),
        ];
        for v in &variants {
            let hasher = v.create_hasher();
            assert!(std::mem::size_of_val(&hasher) > 0);
        }
    }

    #[test]
    fn test_hash_algo_to_hash_type() {
        assert_eq!(HashAlgo::Sha256.to_hash_type(None).unwrap(), HashType::SHA256);
        assert_eq!(HashAlgo::Blake3.to_hash_type(Some(64)).unwrap(), HashType::BLAKE3(64));
        assert_eq!(HashAlgo::Shake128.to_hash_type(None).unwrap(), HashType::SHAKE128(32));
    }

    #[test]
    fn test_hash_type_suffix() {
        assert_eq!(HashType::SHA256.suffix(), "sha256");
        assert_eq!(HashType::SHAKE128(32).suffix(), "shake128-256");
        assert_eq!(HashType::BLAKE3(64).suffix(), "blake3-512");
    }

    #[test]
    fn test_from_suffix_roundtrip() {
        for algo in &[
            HashAlgo::Md5,
            HashAlgo::Sha1,
            HashAlgo::Sha256,
            HashAlgo::Sha3512,
            HashAlgo::Blake2b512,
        ] {
            let ht = algo.to_hash_type(None).unwrap();
            let recovered = HashAlgo::from_suffix(&ht.suffix()).unwrap();
            assert_eq!(&recovered, algo);
        }
    }
}
