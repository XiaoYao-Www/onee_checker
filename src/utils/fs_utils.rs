// 檔案系統函式庫
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{self, Write};


use crate::types::Hash::{HashData};
use crate::types::FS::FileNode;


/// ### 列出所有檔案
///
/// 列出輸入路徑所代表的所有檔案。
///
/// - path 目標路徑
pub fn list_file(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = Vec::new();

    if !path.is_dir() {
        // 檔案類型
        entries.push(path.to_path_buf());
        return Ok(entries);
    }

    for entry in fs::read_dir(path)? {
        // 資料夾類型
        let entry: fs::DirEntry = entry?;
        entries.push(entry.path());
    }

    // 依照名稱排序
    entries.sort_by(|a: &PathBuf, b: &PathBuf| {
        let a_name: String = a
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        let b_name: String = b
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        a_name.cmp(&b_name)
    });

    Ok(entries)
}

/// ### 創建檔案節點
///
/// 根據指定路徑創建節點結構。
///
/// - path 路徑
pub fn build_file_node(path: &Path) -> io::Result<FileNode> {
    let metadata: fs::Metadata = fs::metadata(path)?;
    let name: String = path
        .file_name()
        .map_or(path.display().to_string(), |n: &std::ffi::OsStr| {
            n.to_string_lossy().into_owned()
        });

    // 將 SystemTime 轉成 UNIX timestamp，如果失敗就用 0
    let last_modified: u64 = metadata
        .modified()
        .ok()
        .and_then(|t: SystemTime| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d: std::time::Duration| d.as_secs());

    let is_dir: bool = path.is_dir();

    // 處理子節點
    let children: Option<Vec<FileNode>> = if is_dir {
        let mut list: Vec<FileNode> = fs::read_dir(path)?
            .filter_map(|entry: Result<fs::DirEntry, io::Error>| entry.ok())
            .map(|entry: fs::DirEntry| build_file_node(&entry.path()))
            .filter_map(|res: Result<FileNode, io::Error>| res.ok())
            .collect();

        // 按名稱排序
        list.sort_by_key(|n: &FileNode| n.name.to_lowercase());
        Some(list)
    } else {
        None
    };

    // 計算大小
    let size: u64 = if is_dir {
        children.as_ref().map_or(0, |nodes: &Vec<FileNode>| nodes.iter().map(|n: &FileNode| n.size).sum())
    } else {
        metadata.len()
    };

    Ok(FileNode {
        name,
        is_dir,
        size,
        last_modified,
        children,
    })
}

/// ### 儲存雜湊結果至檔案
///
/// 將雜湊結果寫入指定檔案
/// 格式為: <hash> *<relative_path>
///
/// - data 雜湊資料列表
/// - outputFile 輸出檔案路徑
pub fn save_hash_to_file(data: &[HashData], output_file: &Path) -> io::Result<()> {
    let mut file: File = File::create(output_file)?;
    for entry in data {
        // 轉換路徑分隔符為 /
        let path_str: String = entry.path.to_string_lossy().replace('\\', "/");
        // 寫入驗證資訊，使用 * 表示二進位模式
        writeln!(file, "{} *{}", entry.hash_hex(), path_str)?;
    }
    Ok(())
}