// TXT 紀錄檔系統
use chrono::{Local, TimeZone};
use clap::ValueEnum;
// 1. 為了避免衝突，將 fmt::Write 改名或只在需要時局部引入
use std::fmt::{Write as FmtWrite}; 
// 2. 這是操作檔案必須的
use std::io::{self, Write as IoWrite}; 
use std::path::Path;

use crate::types::fs_types::FileNode;
use crate::utils::FS::build_file_node;

// ========== 類型定義 ==========

/// ### 檔案大小顯示類型
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum TreeStringSizeType {
    /// 1024 為底
    Binary,
    /// 1000 為底
    Decimal,
    /// 原始位元組數 (Bytes)
    Raw,
}

impl TreeStringSizeType {
    /// ### 格式化一個數值
    ///
    /// - bytes 位元組大小
    pub fn format(&self, bytes: u64) -> String {
        match self {
            TreeStringSizeType::Raw => {
                let s: String = bytes.to_string();
                let mut result: String = String::new();
                let len: usize = s.len();

                for (i, c) in s.chars().enumerate() {
                    result.push(c);
                    // 如果剩餘長度是 3 的倍數，且不是最後一個數字，就加逗點
                    if (len - i - 1) % 3 == 0 && i != len - 1 {
                        result.push(',');
                    }
                }
                format!("{} B", result)
            }
            TreeStringSizeType::Binary => {
                self.format_with_base(bytes, 1024.0, &["B", "KiB", "MiB", "GiB", "TiB", "PiB"])
            }
            TreeStringSizeType::Decimal => {
                self.format_with_base(bytes, 1000.0, &["B", "KB", "MB", "GB", "TB", "PB"])
            }
        }
    }

    fn format_with_base(&self, bytes: u64, base: f64, units: &[&str]) -> String {
        if bytes == 0 {
            return format!("0 {}", units[0]);
        }

        let bytes_f: f64 = bytes as f64;
        let i: usize = (bytes_f.log(base).floor() as usize).min(units.len() - 1);

        if i == 0 {
            format!("{} {}", bytes, units[i])
        } else {
            let value: f64 = bytes_f / base.powi(i as i32);
            format!("{:.2} {}", value, units[i])
        }
    }
}

#[derive(Clone)]
pub struct TreeStringOption {
    pub size: Option<TreeStringSizeType>,
    pub last_modified: bool,
    pub created_at: bool,
}

// ========== 函式定義 ==========

pub fn format_unix_to_local(secs: i64) -> String {
    // 使用 Local 時區進行轉換
    // timestamp_opt(秒, 奈秒)
    match Local.timestamp_opt(secs, 0) {
        chrono::LocalResult::Single(dt) => {
            // %Y-%m-%d %H:%M:%S : 年-月-日 時:分:秒
            // %Z : 時區簡稱 (例如 CST)
            // %:z : 時區偏移 (例如 +08:00)
            dt.format("%Y-%m-%d %H:%M:%S %Z").to_string()
        }
        chrono::LocalResult::Ambiguous(dt1, _dt2) => {
            // 極少數情況下（如日光節約時間調整）會有兩個可能時間
            dt1.format("%Y-%m-%d %H:%M:%S %Z").to_string()
        }
        chrono::LocalResult::None => "無效的時間戳".to_string(),
    }
}

/// ### 取得 TREE 字串
pub fn write_tree_to<W: IoWrite>(
    dest: &mut W,
    path: &Path,
    option: &TreeStringOption
) -> io::Result<()> {
    // 這裡 build_file_node 的邏輯不變
    let root_node: FileNode = build_file_node(path)?;
    
    let root_info: String = format_node_info(&root_node, option);

    writeln!(dest, "# ******************************")?;
    writeln!(dest, "# Generation Time: {}", Local::now().format("%Y-%m-%d %H:%M:%S %z"))?;
    writeln!(dest, "# ******************************")?;
    writeln!(dest, "")?;
    
    // 當 W 是 io::Write 時，writeln! 會傳回 io::Result，
    // 這與你的函式回傳值類型相符，所以可以使用 ? 運算子。
    writeln!(dest, "{}{}", root_node.name, root_info)?;

    if let Some(children) = root_node.children {
        write_tree_recursive(dest, &children, "", option)?;
    }
    
    Ok(())
}

/// ### 統一格式化節點資訊 (大小、建立、修改)
fn format_node_info(node: &FileNode, option: &TreeStringOption) -> String {
    let mut info = String::new();

    // 處理大小
    if let Some(size_type) = &option.size {
        write!(info, " ({})", size_type.format(node.size)).unwrap();
    }

    // 處理建立時間
    if option.created_at {
        if let Some(t) = node.created_at {
            write!(info, " [cr: {}]", format_unix_to_local(t)).unwrap();
        }
    }

    // 處理修改時間
    if option.last_modified {
        if let Some(t) = node.last_modified {
            write!(info, " [mod: {}]", format_unix_to_local(t)).unwrap();
        }
    }

    info
}

/// ### 遞迴建構 Tree
fn write_tree_recursive<W: IoWrite>(
    dest: &mut W,
    nodes: &Vec<FileNode>, // 或是 &Vec<FileNode> 以減少 clone
    prefix: &str,
    option: &TreeStringOption,
) -> io::Result<()> {
    let mut iter = nodes.iter().peekable();

    while let Some(node) = iter.next() {
        let is_last = iter.peek().is_none();
        let connector = if is_last { "└── " } else { "├── " };
        let node_info = format_node_info(node, option);

        // 這裡的 writeln! 現在操作的是 io::Write，支援 ?
        writeln!(dest, "{}{}{}{}", prefix, connector, node.name, node_info)?;

        if let Some(ref children) = node.children {
            let new_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            write_tree_recursive(dest, &children.clone(), &new_prefix, option)?;
        }
    }
    Ok(())
}