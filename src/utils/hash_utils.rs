// 哈希相關函式庫
use std::io::{self, Read};

use crate::types::Hash::{DynHasher, HashType};


/// ### 計算單一檔案的雜湊值
///
/// 使用雜湊算法計算檔案的雜湊值。
///
/// - data 檔案讀取器
/// - hasher 雜湊算法器
/// - bufferSize 緩衝區大小
pub fn compute_hash_reader<R: Read>(
    mut data: R,
    mut hasher: Box<dyn DynHasher>,
    buffer: &mut [u8]
) -> io::Result<Vec<u8>> {
    loop {
        let n: usize = data.read(buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize())
}

/// ### 哈希副檔名
///
/// 取得哈希類型的對應副檔名。
///
/// - hashType 哈希類型
pub fn hash_suffix(hash_type: &HashType) -> String {
    match hash_type {
        HashType::MD5 => "md5".into(),
        HashType::SHA1 => "sha1".into(),
        HashType::SHA224 => "sha224".into(),
        HashType::SHA256 => "sha256".into(),
        HashType::SHA384 => "sha384".into(),
        HashType::SHA512 => "sha512".into(),
        HashType::SHA3_256 => "sha3_256".into(),
        HashType::SHA3_512 => "sha3_512".into(),
        HashType::SHAKE128(len) => format!("shake128-{}", len * 8),
        HashType::SHAKE256(len) => format!("shake256-{}", len * 8),
        HashType::BLAKE2S256 => "blake2s256".into(),
        HashType::BLAKE2B512 => "blake2b512".into(),
        HashType::BLAKE3(len) => format!("blake3-{}", len * 8),
    }
}
