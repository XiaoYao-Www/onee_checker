use crate::hash_utils;

use chrono::{DateTime, Local};
use serde::Serialize;
use std::fs;
use std::io;
use std::path::Path;

/// 檔案/資料夾項目資料結構
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
}

/// ### 列出目標資料夾下的所有檔案
///
/// 會跳過隱藏檔案
///
/// - path 目標資料夾路徑
pub fn list_dir(path: &Path) -> io::Result<Vec<FileEntry>> {
    let mut entries: Vec<FileEntry> = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry: fs::DirEntry = entry?;
        let file_name: String = entry.file_name().to_string_lossy().into_owned();
        if file_name.starts_with('.') {
            continue;
        } // 跳過隱藏檔案
        let metadata: fs::Metadata = entry.metadata()?;

        entries.push(FileEntry {
            name: file_name,
            is_dir: metadata.is_dir(),
        });
    }

    // 依照名稱排序
    entries
        .sort_by(|a: &FileEntry, b: &FileEntry| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

/// 用於 JSON 序列化的節點結構
#[derive(Serialize)]
struct Node {
    name: String,
    is_dir: bool,
    size: Option<u64>,
    sha: Option<String>,
    last_modified: Option<String>,
    children: Option<Vec<Node>>,
}

/// ### 創建Node
///
/// 根據指定路徑創建node結構
///
/// - path 路徑
fn build_node(path: &Path) -> io::Result<Node> {
    let metadata: fs::Metadata = fs::metadata(path)?;
    let name: String = path
        .file_name()
        .map_or(path.display().to_string(), |n: &std::ffi::OsStr| {
            n.to_string_lossy().into_owned()
        });

    let last_modified: Option<String> = metadata.modified().ok().map(|t| {
        let dt: DateTime<Local> = t.into();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    });

    let is_dir: bool = path.is_dir();

    let sha: Option<String> = if !is_dir {
        let sha_result: (bool, Vec<hash_utils::ShaData>) = hash_utils::hash_selected(path)?;
        if !sha_result.0 {
            Some(sha_result.1[0].sha.clone())
        } else {
            None
        }
    } else {
        None
    };

    let children: Option<Vec<Node>> = if is_dir {
        let mut list: Vec<Node> = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry: fs::DirEntry = entry?;
            let child: Node = build_node(&entry.path())?;
            list.push(child);
        }
        list.sort_by(|a: &Node, b: &Node| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Some(list)
    } else {
        None
    };

    let size: Option<u64> = if is_dir {
        if children.is_some() {
            let mut size: u64 = 0;
            for child in children.as_ref().unwrap() {
                size += child.size.unwrap_or(0);
            }
            Some(size)
        } else {
            None
        }
    } else {
        Some(metadata.len())
    };

    Ok(Node {
        name,
        is_dir,
        size,
        sha,
        last_modified,
        children,
    })
}

/// 以 JSON 形式輸出目錄結構
pub fn get_json_string(path: &Path) -> io::Result<String> {
    // 遞迴建構目錄/檔案節點
    let node: Node = build_node(path)?;
    // 使用 serde_json 將節點轉為漂亮的 JSON 字串
    let json_str: String = serde_json::to_string_pretty(&node)?;
    Ok(json_str)
}

// / 以文字型式輸出目錄結構樹
// pub fn print_txt_tree(path: &Path) -> io::Result<()> {
//     // 輔助函式：遞迴列印子目錄
//     fn print_helper(path: &Path, indent: usize) -> io::Result<()> {
//         for entry in fs::read_dir(path)? {
//             let entry = entry?;
//             let file_name = entry.file_name().to_string_lossy().into_owned();
//             let child_path = entry.path();
//             if child_path.is_dir() {
//                 println!("{:indent$}+ {}/", "", file_name, indent = indent);
//                 print_helper(&child_path, indent + 2)?;
//             } else {
//                 println!("{:indent$}- {}", "", file_name, indent = indent);
//             }
//         }
//         Ok(())
//     }

//     if path.is_dir() {
//         // 如果選到的是資料夾，先印出資料夾名稱，再印內容
//         println!("{}:", path.display());
//         print_helper(path, 2)?;
//     } else {
//         // 如果是單一檔案，直接印出檔案路徑
//         println!("{}", path.display());
//     }
//     Ok(())
// }
