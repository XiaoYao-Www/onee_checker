// hash_utils.rs

use std::fs::File;
use std::io::{self, Read};
use sha2::{Sha256, Digest};

/// 計算選取檔案或資料夾的 SHA256 並輸出
pub fn hash_selected(path: &std::path::Path) -> io::Result<()> {
    if path.is_dir() {
        // 對目錄下的所有檔案遞迴計算 SHA256
        let mut file_list: Vec<std::path::PathBuf> = Vec::new();
        collect_files(path, &mut file_list)?;
        file_list.sort(); // 排序，以確保一致性
        for file_path in file_list {
            if file_path.is_file() {
                let hash = compute_file_hash(&file_path)?;
                // 輸出格式符合 sha256sum，可使用 sha256sum --check 驗證:contentReference[oaicite:2]{index=2}
                println!("{} *{}", hash, file_path.display());
            }
        }
    } else if path.is_file() {
        // 如果是單一檔案
        let hash = compute_file_hash(path)?;
        println!("{} *{}", hash, path.display());
    }
    Ok(())
}

/// 遞迴收集目錄底下的所有檔案路徑
fn collect_files(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

/// 計算單一檔案的 SHA256 雜湊值
fn compute_file_hash(path: &std::path::Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 4096];
    // 讀取檔案內容並更新雜湊器
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
