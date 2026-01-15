// fs_utils.rs

use std::fs;
use std::path::Path;
use std::io;
use serde::Serialize;

/// 檔案/資料夾項目資料結構
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
}

/// 列出指定目錄下的檔案和資料夾
pub fn list_dir(path: &Path) -> io::Result<Vec<FileEntry>> {
    let mut entries: Vec<FileEntry> = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if file_name.starts_with('.') { continue; } // 跳過隱藏檔案
        let metadata = entry.metadata()?;
        entries.push(FileEntry {
            name: file_name,
            is_dir: metadata.is_dir(),
        });
    }
    // 依照名稱排序
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

/// 用於 JSON 序列化的節點結構
#[derive(Serialize)]
struct Node {
    name: String,
    is_dir: bool,
    children: Option<Vec<Node>>,
}

/// 以 JSON 形式輸出目錄結構
pub fn print_json_tree(path: &Path) -> io::Result<()> {
    // 遞迴建構目錄/檔案節點
    fn build_node(path: &Path) -> io::Result<Node> {
        let name = path.file_name()
            .map_or(path.display().to_string(), |n| n.to_string_lossy().into_owned());
        let is_dir = path.is_dir();
        let children = if is_dir {
            let mut list = Vec::new();
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let child = build_node(&entry.path())?;
                list.push(child);
            }
            Some(list)
        } else {
            None
        };
        Ok(Node { name, is_dir, children })
    }

    let node = build_node(path)?;
    // 使用 serde_json 將節點轉為漂亮的 JSON 字串
    let json_str = serde_json::to_string_pretty(&node).expect("JSON 序列化失敗");
    println!("{}", json_str);
    Ok(())
}

/// 以文字型式輸出目錄結構樹
pub fn print_txt_tree(path: &Path) -> io::Result<()> {
    // 輔助函式：遞迴列印子目錄
    fn print_helper(path: &Path, indent: usize) -> io::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().into_owned();
            let child_path = entry.path();
            if child_path.is_dir() {
                println!("{:indent$}+ {}/", "", file_name, indent=indent);
                print_helper(&child_path, indent + 2)?;
            } else {
                println!("{:indent$}- {}", "", file_name, indent=indent);
            }
        }
        Ok(())
    }

    if path.is_dir() {
        // 如果選到的是資料夾，先印出資料夾名稱，再印內容
        println!("{}:", path.display());
        print_helper(path, 2)?;
    } else {
        // 如果是單一檔案，直接印出檔案路徑
        println!("{}", path.display());
    }
    Ok(())
}
