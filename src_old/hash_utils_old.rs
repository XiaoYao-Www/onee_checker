// 哈希驗證相關函數與結構
use crate::type_define::{HashData, HashType};
use blake2::{Blake2b512, Blake2s256};
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use sha3::{Sha3_256, Sha3_512, Shake128, Shake256};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use crate::type_define;

// ========== 類型定義 ==========



// ========== 函式定義 ==========



/// ### 驗證單個檔案的哈希
pub fn checkFileHash(path: &Path, expected_hash: &[u8], hash_type: &HashType) -> io::Result<bool> {
    let computed = computeFileHash(path, hash_type)?;
    Ok(computed == expected_hash)
}



/// ### 依據指定類型驗證一個哈希檔案
pub fn checkHashFile(
    hash_file_path: &Path,
    hash_type: HashType,
    progress_sender: Sender<type_define::TaskProgress>,
) -> io::Result<Vec<(PathBuf, bool)>> {
    let file = File::open(hash_file_path)?;
    let reader = BufReader::new(file);
    let base_dir = hash_file_path.parent().unwrap_or(Path::new("."));

    let mut results = Vec::new();
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    let total = lines.len();

    let _ = progress_sender.send(type_define::TaskProgress::Hash {
        total,
        current: None,
    });

    for (i, line) in lines.iter().enumerate() {
        let _ = progress_sender.send(type_define::TaskProgress::Hash {
            total,
            current: Some(i + 1),
        });

        let line = line.trim();
        if line.is_empty() { continue; }

        // 解析 hash *filename
        if let Some((hash_str, file_part)) = line.split_once(' ') {
            let file_part = file_part.trim_start();
            let file_path_str = if let Some(stripped) = file_part.strip_prefix('*') {
                stripped
            } else {
                file_part
            };

            let target_path = base_dir.join(file_path_str);
            
            if let Ok(expected_bytes) = hex::decode(hash_str) {
                let is_valid = checkFileHash(&target_path, &expected_bytes, &hash_type).unwrap_or(false);
                results.push((PathBuf::from(file_path_str), is_valid));
            } else {
                results.push((PathBuf::from(file_path_str), false));
            }
        }
    }

    Ok(results)
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, files)?;
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

/// ### 計算單一檔案的雜湊值
///
/// 使用雜湊算法計算檔案的雜湊值
///
/// - path 檔案路徑
/// - hashType 算法類型
pub fn computeFileHash(path: &Path, hash_type: &HashType) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = [0u8; 8192];

    match hash_type {
        HashType::MD5 => {
            let mut hasher = Md5::new();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashType::SHA1 => {
            let mut hasher = Sha1::new();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashType::SHA224 => {
            let mut hasher = Sha224::new();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashType::SHA256 => {
            let mut hasher = Sha256::new();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashType::SHA384 => {
            let mut hasher = Sha384::new();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashType::SHA512 => {
            let mut hasher = Sha512::new();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashType::SHA3256 => {
            let mut hasher = Sha3_256::new();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashType::SHA3512 => {
            let mut hasher = Sha3_512::new();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashType::SHAKE128(size) => {
            use sha3::digest::{Update, ExtendableOutput, XofReader};
            let mut hasher = Shake128::default();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            let mut reader = hasher.finalize_xof();
            let mut output = vec![0u8; *size];
            XofReader::read(&mut reader, &mut output);
            Ok(output.to_vec())
        }
        HashType::SHAKE256(size) => {
            use sha3::digest::{Update, ExtendableOutput, XofReader};
            let mut hasher = Shake256::default();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            let mut reader = hasher.finalize_xof();
            let mut output = vec![0u8; *size];
            XofReader::read(&mut reader, &mut output);
            Ok(output.to_vec())
        }
        HashType::BLAKE2 => {
            let mut hasher = Blake2b512::new();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashType::BLAKE3 => {
            let mut hasher = blake3::Hasher::new();
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 { break; }
                hasher.update(&buffer[..n]);
            }
            Ok(hasher.finalize().as_bytes().to_vec())
        }
    }
}

/// ### 計算選取檔案或資料夾的雜湊值
///
/// 目前用sha256，回傳(是否為資料夾, 雜湊結果)
///
/// - path 檔案或資料夾路徑
/// - progress_sender 用於回報進度的發送端
pub fn calaPathHash(
    path: &Path,
    hashType: HashType,
    progress_sender: Sender<type_define::TaskProgress>,
) -> io::Result<Vec<HashData>> {
    let mut result: Vec<HashData> = Vec::new();

    // 🔑 以輸入路徑的父目錄作為相對根
    let base_dir: &Path = path.parent().unwrap_or(path);

    if path.is_dir() {
        let mut files: Vec<PathBuf> = Vec::new();
        collect_files(path, &mut files)?;
        files.sort();

        let total_files = files.len();
        let _ = progress_sender.send(type_define::TaskProgress::Hash {
            total: total_files,
            current: None,
        });

        for (i, file_path) in files.iter().enumerate() {
            if file_path.is_file() {
                let _ = progress_sender.send(type_define::TaskProgress::Hash {
                    total: total_files,
                    current: Some(i + 1),
                });

                let hash_bytes = computeFileHash(file_path, &hashType)?;

                // 相對於輸入路徑的父目錄
                let relative_path: PathBuf = file_path
                    .strip_prefix(base_dir)
                    .map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("path {:?} is not under base dir {:?}", file_path, base_dir),
                        )
                    })?
                    .to_path_buf();

                result.push(HashData {
                    path: relative_path,
                    hash_bytes: hash_bytes,
                    hash_type: hashType,
                });
            }
        }

        Ok(result)
    } else if path.is_file() {
        let _ = progress_sender.send(type_define::TaskProgress::Hash {
            total: 1,
            current: None,
        });
        let _ = progress_sender.send(type_define::TaskProgress::Hash {
            total: 1,
            current: Some(1),
        });

        let hash_bytes = computeFileHash(path, &hashType)?;

        let relative_path = path
            .strip_prefix(base_dir)
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("path {:?} is not under base dir {:?}", path, base_dir),
                )
            })?
            .to_path_buf();

        result.push(HashData {
            path: relative_path,
            hash_bytes: hash_bytes,
            hash_type: hashType,
        });

        Ok(result)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "path is neither file nor directory",
        ))
    }
}


