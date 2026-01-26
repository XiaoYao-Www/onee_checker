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
/// - path 目標資料夾路徑
pub fn list_dir(path: &Path) -> io::Result<Vec<FileEntry>> {
    let mut entries: Vec<FileEntry> = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry: fs::DirEntry = entry?;
        let file_name: String = entry.file_name().to_string_lossy().into_owned();
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
    size: u64,
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

    let size: u64 = if is_dir {
        children
            .as_ref()
            .map_or(0, |nodes| nodes.iter().map(|n| n.size).sum())
    } else {
        metadata.len()
    };

    Ok(Node {
        name,
        is_dir,
        size,
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

/// 以文字型式輸出目錄結構樹
pub fn get_tree_string(path: &Path) -> io::Result<String> {
    let mut tree_string = String::new();
    if let Some(name) = path.file_name() {
        tree_string.push_str(&name.to_string_lossy());
        tree_string.push('\n');
    }
    build_tree_string_recursive(path, &mut tree_string, "")?;
    Ok(tree_string)
}

/// 遞迴建構文字樹的輔助函式
fn build_tree_string_recursive(
    dir: &Path,
    tree_string: &mut String,
    prefix: &str,
) -> io::Result<()> {
    let entries = list_dir(dir)?; // 重用 list_dir 來取得排序且過濾後的列表
    let mut iter = entries.iter().peekable();
    while let Some(entry) = iter.next() {
        let is_last = iter.peek().is_none();
        let connector = if is_last { "└── " } else { "├── " };
        tree_string.push_str(&format!("{}{}{}\n", prefix, connector, entry.name));

        if entry.is_dir {
            let new_prefix = if is_last { "    " } else { "│   " };
            let child_path = dir.join(&entry.name);
            build_tree_string_recursive(&child_path, tree_string, &format!("{}{}", prefix, new_prefix))?;
        }
    }
    Ok(())
}
