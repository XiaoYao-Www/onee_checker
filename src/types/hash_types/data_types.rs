// 哈希資料類型定義
use super::hasher::{
    DynHasher,
    Md5Hasher,
    Sha1Hasher,
    Sha224Hasher,
    Sha256Hasher,
    Sha384Hasher,
    Sha512Hasher,
    Sha3_256Hasher,
    Sha3_512Hasher,
    Shake128XofHasher,
    Shake256XofHasher,
    Blake2s256Hasher,
    Blake2b512Hasher,
    Blake3Hasher,
};
use std::path::PathBuf;


/// ### 哈希驗證類型
/// 
/// 所有支援的哈希驗證類型。
/// 
/// 長度參數以位元組為單位。
/// 
/// - create_hasher() 建立對應的哈希器實例
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HashType {
    // MD5
    MD5,
    // SHA-1
    SHA1,
    // SHA-2
    SHA224,
    SHA256,
    SHA384,
    SHA512,
    // SHA-3
    SHA3_256,
    SHA3_512,
    // SHAKE
    SHAKE128(usize),
    SHAKE256(usize),
    // BLAKE
    BLAKE2S256,
    BLAKE2B512,
    BLAKE3(usize),
}

impl HashType {
    pub fn create_hasher(&self) -> Box<dyn DynHasher> {
        match self {
            HashType::MD5 => Box::new(Md5Hasher::new()),
            HashType::SHA1 => Box::new(Sha1Hasher::new()),
            HashType::SHA224 => Box::new(Sha224Hasher::new()),
            HashType::SHA256 => Box::new(Sha256Hasher::new()),
            HashType::SHA384 => Box::new(Sha384Hasher::new()),
            HashType::SHA512 => Box::new(Sha512Hasher::new()),
            HashType::SHA3_256 => Box::new(Sha3_256Hasher::new()),
            HashType::SHA3_512 => Box::new(Sha3_512Hasher::new()),
            HashType::SHAKE128(output_size) => Box::new(Shake128XofHasher::new(*output_size)),
            HashType::SHAKE256(output_size) => Box::new(Shake256XofHasher::new(*output_size)),
            HashType::BLAKE2S256 => Box::new(Blake2s256Hasher::new()),
            HashType::BLAKE2B512 => Box::new(Blake2b512Hasher::new()),
            HashType::BLAKE3(output_size) => Box::new(Blake3Hasher::new(*output_size)),
        }
    }
}

/// ### Hash資料結構
///
/// - path 檔案路徑
/// - hash_bytes 雜湊值
/// - hash_type 雜湊類型
/// - hash_hex() 雜湊字串
pub struct HashData {
    pub path: PathBuf,
    pub hash_bytes: Vec<u8>,
    pub hash_type: HashType,
}

impl HashData {
    pub fn hash_hex(&self) -> String {
        hex::encode(&self.hash_bytes)
    }
}