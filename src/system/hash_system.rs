// 哈希功能建構
use std::fs::File;
use std::io::{self};
use std::path::Path;
use rayon::ThreadPoolBuilder;

use crate::types::Hash::{DynHasher, HashData, HashType};
use crate::utils::hash_utils::compute_hash_reader;

/// ### 計算單一檔案的雜湊值
///
/// 使用雜湊算法計算檔案的雜湊值。
///
/// - path 檔案路徑
/// - hash_type 雜湊類型
/// - buffer 緩衝區
pub fn compute_file_hash(
    path: &Path,
    hash_type: &HashType,
    buffer: &mut [u8],
) -> io::Result<Vec<u8>> {
    if path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "輸入路徑為目錄，應當為檔案。",
        ));
    }
    let mut file: File = File::open(path)?;
    let hasher: Box<dyn DynHasher> = hash_type.create_hasher();
    compute_hash_reader(&mut file, hasher, buffer)
}

/// ### 檢查檔案雜湊值
///
/// 檢查檔案的雜湊值是否與預期值相符。
///     
/// - path 檔案路徑
/// - expected_hash 預期雜湊值
/// - hash_type 雜湊類型
/// - buffer 緩衝區
pub fn check_file_hash(
    path: &Path,
    expected_hash: &[u8],
    hash_type: &HashType,
    buffer: &mut [u8],
) -> io::Result<bool> {
    if path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "輸入路徑為目錄，應當為檔案。",
        ));
    }
    let computed_hash: Vec<u8> = compute_file_hash(path, hash_type, buffer)?;
    Ok(computed_hash.as_slice() == expected_hash)
}