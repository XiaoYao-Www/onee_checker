// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Hash 計算引擎 — 單檔計算、並行計算、驗證。

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::algorithm::{HashData, HashType};
use crate::error::OneeError;
use crate::fs::{parse_hash_file, canonicalize_root, sanitize_rel_path};
use crate::hasher::HasherEnum;

/// 從 `Read` 來源計算 hash（streaming）。
///
/// `buffer` 是外部傳入的可覆用緩衝區（避免重複配置）。
/// `hasher` 為 mutable reference，呼叫方可復用 hasher 多次計算。
pub fn compute_hash_reader<R: Read>(
    mut reader: R,
    hasher: &mut HasherEnum,
    buffer: &mut [u8],
) -> io::Result<Vec<u8>> {
    loop {
        let n = reader.read(buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hasher.clone().finalize())
}

/// 計算單一檔案的 hash 值。
pub fn compute_file_hash(
    path: &Path,
    hash_type: &HashType,
    buffer: &mut [u8],
) -> io::Result<Vec<u8>> {
    if path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("路徑為目錄，跳過: {}", path.display()),
        ));
    }
    let file = File::open(path)?;
    let mut hasher = hash_type.create_hasher();
    compute_hash_reader(file, &mut hasher, buffer)
}

/// 並行計算多個檔案的 hash 值。
///
/// 在 `map_init` 中 clone hasher template 供每個檔案使用，
/// 避免逐檔重建（clone 成本約 64–200 bytes copy）。
///
/// `buffer_size` 為每個 worker 執行緒的讀取緩衝區大小（bytes）。
pub fn compute_hashes_parallel(
    files: &[PathBuf],
    hash_type: &HashType,
    buffer_size: usize,
) -> Vec<std::result::Result<HashData, OneeError>> {
    files
        .par_iter()
        .map_init(
            || (vec![0u8; buffer_size], hash_type.create_hasher()),
            |(buf, template), path| {
                let mut hasher = template.clone();
                compute_hash_reader(File::open(path)?, &mut hasher, buf)
                    .map(|bytes| HashData::new(path.to_path_buf(), bytes, hash_type.clone()))
                    .map_err(|e| OneeError::Io(e))
            },
        )
        .collect()
}

/// 驗證單一檔案的 hash 是否符合預期。
pub fn verify_file_hash(
    path: &Path,
    expected_hex: &str,
    hash_type: &HashType,
    buffer: &mut [u8],
) -> std::result::Result<bool, OneeError> {
    let computed = compute_file_hash(path, hash_type, buffer)?;
    let computed_hex = hex::encode(&computed);
    Ok(computed_hex == expected_hex)
}

/// 驗證 hash 檔中的所有項目。
///
/// 回傳 (path, is_match, actual_hash_hex) 的列表。
///
/// # 安全性
///
/// - 根目錄會先 `canonicalize_root` 鎖定
/// - 每個相對路徑會通過 `sanitize_rel_path` 防止 path traversal
pub fn verify_hash_file(
    hash_file_path: &Path,
    hash_type: &HashType,
    root_dir: &Path,
    buffer_size: usize,
) -> Vec<std::result::Result<(PathBuf, bool, String), OneeError>> {
    let content = match std::fs::read_to_string(hash_file_path) {
        Ok(c) => c,
        Err(e) => return vec![Err(OneeError::Io(e))],
    };

    // 安全地正規化根目錄
    let safe_root = match canonicalize_root(root_dir) {
        Ok(r) => r,
        Err(e) => return vec![Err(e)],
    };

    let entries = parse_hash_file(&content);

    entries
        .par_iter()
        .map_init(
            || vec![0u8; buffer_size],
            |buf, entry| {
                // 消毒相對路徑，防止 path traversal
                let full_path = match sanitize_rel_path(&entry.rel_path, &safe_root) {
                    Ok(p) => p,
                    Err(e) => return Err(e),
                };
                match verify_file_hash(&full_path, &entry.hash_hex, hash_type, buf) {
                    Ok(true) => Ok((full_path, true, entry.hash_hex.clone())),
                    Ok(false) => {
                        let actual = compute_file_hash(&full_path, hash_type, buf)
                            .map(|b| hex::encode(&b))
                            .unwrap_or_else(|_| "ERR".into());
                        Ok((full_path, false, actual))
                    }
                    Err(e) => Err(e),
                }
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::fs;

    #[test]
    fn test_compute_hash_reader_sha256() {
        let data = b"hello world";
        let hash_type = HashType::SHA256;
        let mut hasher = hash_type.create_hasher();
        let mut buf = vec![0u8; 4096];

        let result = compute_hash_reader(&data[..], &mut hasher, &mut buf).unwrap();
        let hex = hex::encode(&result);

        // SHA256("hello world") = b94d27b9934d3e08a52e52d7da7dabfac4e8cde9
        assert_eq!(hex, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_compute_file_hash() {
        let dir = std::env::temp_dir().join("onee_test_hash");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let file_path = dir.join("test.bin");
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"test data for hash").unwrap();
        drop(f);

        let mut buf = vec![0u8; 4096];
        let result = compute_file_hash(&file_path, &HashType::MD5, &mut buf).unwrap();
        assert_eq!(result.len(), 16); // MD5 是 16 bytes

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_verify_file_hash_match() {
        let dir = std::env::temp_dir().join("onee_test_verify");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let file_path = dir.join("hello.txt");
        fs::write(&file_path, b"hello").unwrap();

        let mut buf = vec![0u8; 4096];
        let hash = compute_file_hash(&file_path, &HashType::SHA256, &mut buf).unwrap();
        let hex = hex::encode(&hash);

        let ok = verify_file_hash(&file_path, &hex, &HashType::SHA256, &mut buf).unwrap();
        assert!(ok);

        let bad = verify_file_hash(&file_path, "000000", &HashType::SHA256, &mut buf).unwrap();
        assert!(!bad);

        let _ = fs::remove_dir_all(&dir);
    }
}
