// 檔案操作相關函數與結構
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::io;
use std::path::{PathBuf, Path};
use std::fmt::Write;
use std::iter::Peekable;
use std::borrow::Cow;
use serde::Serialize;


// ========== 類型定義 ==========



// ========== 函式定義 ==========





/// ### 取得JSON字串
/// 
/// 取得輸入路徑的Json字串。
/// 
/// - path 輸入路徑
pub fn getJsonString(path: &Path) -> io::Result<String> {
    // 遞迴建構目錄/檔案節點
    let node: FileNode = buildFileNode(path)?;
    // 使用 serde_json 將節點轉為漂亮的 JSON 字串
    let json_str: String = serde_json::to_string_pretty(&node)?;
    Ok(json_str)
}

/// ### 取得TREE字串
/// 
/// 取得輸入路徑的Tree字串。
/// 
/// - path 輸入路徑
pub fn getTreeString(path: &Path) -> io::Result<String> {
    let mut tree_string: String = String::new();
    if let Some(name) = path.file_name() {
        tree_string.push_str(&name.to_string_lossy());
        tree_string.push('\n');
    }
    buildTreeStringRecursive(path, &mut tree_string, "")?;
    Ok(tree_string)
}

/// ### 遞迴建構Tree
/// 
/// - dir 路徑
/// - treeString 接續儲存的字串
/// - prefix 接續的前綴
fn buildTreeStringRecursive(
    dir: &Path,
    treeString: &mut String,
    prefix: &str,
) -> io::Result<()> {
    let entries: Vec<PathBuf> = listFile(dir)?; 
    let mut iter: Peekable<std::slice::Iter<'_, PathBuf>> = entries.iter().peekable();
    while let Some(entry) = iter.next() {
        let is_last: bool = iter.peek().is_none();
        let connector: &str = if is_last { "└── " } else { "├── " };
        let file_name: Cow<'_, str> = entry.file_name().unwrap_or_default().to_string_lossy();
        writeln!(treeString, "{}{}{}", prefix, connector, file_name);

        if entry.is_dir() {
            let new_prefix: String = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            buildTreeStringRecursive(entry, treeString, &new_prefix)?;
        }
    }
    Ok(())
}
