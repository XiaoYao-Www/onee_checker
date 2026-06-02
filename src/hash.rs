// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Hash 計算引擎 — 單檔計算、並行計算、驗證。

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use rayon::{prelude::*, ThreadPool};

use crate::algorithm::{HashData, HashType};
use crate::error::OneeError;
use crate::fs::{canonicalize_root, parse_hash_file, sanitize_rel_path};
use crate::hasher::{blake3_hash_bulk, HasherEnum};

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

/// BLAKE3 啟用多線程的最小檔案大小（256 MiB）
const BLAKE3_PARALLEL_THRESHOLD: u64 = 256 * 1024 * 1024;

/// BLAKE3 載入整個檔案的最大大小（1 GiB），避免 OOM
const BLAKE3_BULK_MAX_SIZE: u64 = 1024 * 1024 * 1024;

/// 計算單一檔案的 hash 值。
///
/// # BLAKE3 優化
///
/// 當演算法為 BLAKE3 且檔案大於 256 MiB 但小於 1 GiB 時，
/// 會一次載入整個檔案並使用 `update_bulk()` 啟用 BLAKE3 內部多線程樹狀 hash。
/// 這能將大檔案的 hash 速度提升 3-8x（視 CPU 核心數而定）。
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

    // BLAKE3 大檔案優化 — 一次載入 + 多線程樹狀 hash
    if let HashType::BLAKE3(_) = hash_type {
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();
        if (BLAKE3_PARALLEL_THRESHOLD..=BLAKE3_BULK_MAX_SIZE).contains(&file_size) {
            return compute_blake3_bulk_path(path, hash_type);
        }
    }

    let file = File::open(path)?;
    let mut hasher = hash_type.create_hasher();
    compute_hash_reader(file, &mut hasher, buffer)
}

/// 將整個檔案載入記憶體，使用 `blake3_hash_bulk()` 進行多線程 hash。
///
/// 僅供 `compute_file_hash` 內部呼叫，檔案大小須在 256 MiB ~ 1 GiB 之間。
fn compute_blake3_bulk_path(path: &Path, hash_type: &HashType) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len() as usize;
    let mut data = Vec::with_capacity(file_size);
    file.read_to_end(&mut data)?;
    let out_len = hash_type.default_length();
    Ok(blake3_hash_bulk(&data, out_len))
}

/// 並行計算多個檔案的 hash 值。
///
/// 在 `map_init` 中 clone hasher template 供每個檔案使用，
/// 避免逐檔重建（clone 成本約 64–200 bytes copy）。
///
/// `buffer_size` 為每個 worker 執行緒的讀取緩衝區大小（bytes）。
#[must_use]
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
                    .map(|bytes| HashData::new(path.clone(), bytes, hash_type.clone()))
                    .map_err(OneeError::Io)
            },
        )
        .collect()
}

/// 使用指定 `ThreadPool` 並行計算多個檔案的 hash 值。
///
/// 與 `compute_hashes_parallel` 相同，但使用傳入的 `pool` 而非 global pool。
/// 當呼叫方已建立自訂線程池時使用此版本。
#[must_use]
pub fn compute_hashes_parallel_with_pool(
    pool: &ThreadPool,
    files: &[PathBuf],
    hash_type: &HashType,
    buffer_size: usize,
) -> Vec<std::result::Result<HashData, OneeError>> {
    pool.install(|| {
        files
            .par_iter()
            .map_init(
                || (vec![0u8; buffer_size], hash_type.create_hasher()),
                |(buf, template), path| {
                    let mut hasher = template.clone();
                    compute_hash_reader(File::open(path)?, &mut hasher, buf)
                        .map(|bytes| HashData::new(path.clone(), bytes, hash_type.clone()))
                        .map_err(OneeError::Io)
                },
            )
            .collect()
    })
}

// ──────────────────────────────────────────────────────────
//  多演算法（單次 I/O）
// ──────────────────────────────────────────────────────────

/// 單次 I/O 多演算法並行 hash 計算。
///
/// 讀取每個檔案一次，同時更新多個 hasher（每個演算法一個）。
/// 相比逐個演算法呼叫 `compute_hashes_parallel`，可減少 N-1 倍的 I/O。
///
/// `files` — 要計算的檔案列表
/// `hash_types` — 要計算的演算法列表
/// `buffer_size` — 每個 worker 的讀取緩衝區大小（bytes）
///
/// 回傳 `Vec<Vec<Result<HashData, OneeError>>>`：
/// - 外層 vec：每個演算法一組結果（順序與 `hash_types` 相同）
/// - 內層 vec：該演算法下每個檔案的結果（順序與 `files` 相同）
#[must_use]
pub fn compute_multi_hashes_parallel(
    files: &[PathBuf],
    hash_types: &[HashType],
    buffer_size: usize,
) -> Vec<Vec<std::result::Result<HashData, OneeError>>> {
    if hash_types.is_empty() {
        return vec![];
    }

    // 每個檔案計算所有演算法的結果
    let per_file_results: Vec<Vec<std::result::Result<HashData, OneeError>>> = files
        .par_iter()
        .map_init(
            || {
                let buf = vec![0u8; buffer_size];
                let templates: Vec<HasherEnum> =
                    hash_types.iter().map(HashType::create_hasher).collect();
                (buf, templates)
            },
            |(buf, templates), path| {
                let mut file = match File::open(path) {
                    Ok(f) => f,
                    Err(e) => {
                        let msg = e.to_string();
                        return std::iter::repeat_with(|| {
                            Err(OneeError::Io(std::io::Error::other(msg.clone())))
                        })
                        .take(hash_types.len())
                        .collect();
                    }
                };

                // 從 template clone 出每個演算法的獨立 hasher
                let mut hashers: Vec<HasherEnum> = templates.clone();

                // 一次讀取，同時餵入所有 hasher
                loop {
                    let n = match file.read(buf) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(e) => {
                            let msg = e.to_string();
                            return std::iter::repeat_with(|| {
                                Err(OneeError::Io(std::io::Error::other(msg.clone())))
                            })
                            .take(hash_types.len())
                            .collect();
                        }
                    };
                    for hasher in &mut hashers {
                        hasher.update(&buf[..n]);
                    }
                }

                // 完成所有 hasher
                hash_types
                    .iter()
                    .enumerate()
                    .map(|(i, ht)| {
                        let bytes = hashers[i].clone().finalize();
                        Ok(HashData::new(path.clone(), bytes, ht.clone()))
                    })
                    .collect()
            },
        )
        .collect();

    // 轉置：從 per-file 轉為 per-algorithm
    let num_algos = hash_types.len();
    let mut transposed: Vec<Vec<std::result::Result<HashData, OneeError>>> =
        (0..num_algos).map(|_| Vec::with_capacity(files.len())).collect();

    for file_results in per_file_results {
        for (algo_idx, result) in file_results.into_iter().enumerate() {
            transposed[algo_idx].push(result);
        }
    }

    transposed
}

/// 使用指定 `ThreadPool` 進行單次 I/O 多演算法並行 hash 計算。
///
/// 內部委託給 `compute_multi_hashes_parallel`，但確保 rayon 工作在指定的 pool 內。
#[must_use]
pub fn compute_multi_hashes_parallel_with_pool(
    pool: &ThreadPool,
    files: &[PathBuf],
    hash_types: &[HashType],
    buffer_size: usize,
) -> Vec<Vec<std::result::Result<HashData, OneeError>>> {
    pool.install(|| compute_multi_hashes_parallel(files, hash_types, buffer_size))
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
/// 回傳 (path, `is_match`, `actual_hash_hex`) 的列表。
///
/// # 安全性
///
/// - 根目錄會先 `canonicalize_root` 鎖定
/// - 每個相對路徑會通過 `sanitize_rel_path` 防止 path traversal
#[must_use]
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
                            .map_or_else(|_| "ERR".into(), |b| hex::encode(&b));
                        Ok((full_path, false, actual))
                    }
                    Err(e) => Err(e),
                }
            },
        )
        .collect()
}

/// 使用指定 `ThreadPool` 驗證 hash 檔中的所有項目。
///
/// 與 `verify_hash_file` 相同，但使用傳入的 `pool` 而非 global pool。
#[must_use]
pub fn verify_hash_file_with_pool(
    pool: &ThreadPool,
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

    pool.install(|| {
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
                                .map_or_else(|_| "ERR".into(), |b| hex::encode(&b));
                            Ok((full_path, false, actual))
                        }
                        Err(e) => Err(e),
                    }
                },
            )
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::Write;

    /// 0-byte 空檔案的 hash 測試
    #[test]
    fn test_compute_empty_file() {
        let dir = std::env::temp_dir().join("onee_test_empty_hash");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let empty_file = dir.join("empty.bin");
        fs::write(&empty_file, b"").unwrap();

        let mut buf = vec![0u8; 4096];
        let hash = compute_file_hash(&empty_file, &HashType::SHA256, &mut buf).unwrap();
        // SHA256("") = e3b0c44...
        assert_eq!(
            hex::encode(&hash),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// 10MB 全零資料的串流 hash 測試
    #[test]
    fn test_compute_large_data() {
        let data = vec![0u8; 10 * 1024 * 1024];
        let hash_type = HashType::SHA256;
        let mut hasher = hash_type.create_hasher();
        let mut buf = vec![0u8; 64 * 1024];

        let result = compute_hash_reader(&data[..], &mut hasher, &mut buf).unwrap();
        assert_eq!(result.len(), 32); // SHA256 輸出 32 bytes
    }

    /// 不存在的 hash 檔驗證測試
    #[test]
    fn test_verify_hash_file_missing() {
        let dir = std::env::temp_dir().join("onee_test_verify_missing");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let missing = dir.join("nonexistent.sha256");

        let results = verify_hash_file(&missing, &HashType::SHA256, &dir, 4096);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());

        let _ = fs::remove_dir_all(&dir);
    }

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
