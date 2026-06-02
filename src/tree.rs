// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! 樹狀目錄結構文字輸出。

use std::fmt::Write as FmtWrite;
use std::io::{self, Write as IoWrite};
use std::path::Path;

use chrono::{Local, TimeZone};

use crate::fs::{build_file_node, FileNode};

// ──────────────────────────────────────────────────────────
//  大小顯示格式
// ──────────────────────────────────────────────────────────

/// 檔案大小顯示類型
#[derive(Debug, Clone, Copy)]
pub enum SizeFormat {
    /// 1024 為底 (KiB, MiB, ...)
    Binary,
    /// 1000 為底 (KB, MB, ...)
    Decimal,
    /// 原始 bytes，加逗號分隔
    Raw,
}

impl SizeFormat {
    /// 格式化位元組大小
    pub fn format(&self, bytes: u64) -> String {
        match self {
            Self::Raw => {
                let s = bytes.to_string();
                let mut result = String::new();
                let len = s.len();
                for (i, c) in s.chars().enumerate() {
                    result.push(c);
                    if (len - i - 1) % 3 == 0 && i != len - 1 {
                        result.push(',');
                    }
                }
                format!("{result} B")
            }
            Self::Binary => self.format_with_base(bytes, 1024.0, &["B", "KiB", "MiB", "GiB", "TiB", "PiB"]),
            Self::Decimal => self.format_with_base(bytes, 1000.0, &["B", "KB", "MB", "GB", "TB", "PB"]),
        }
    }

    fn format_with_base(&self, bytes: u64, base: f64, units: &[&str]) -> String {
        if bytes == 0 {
            return format!("0 {}", units[0]);
        }
        let bytes_f = bytes as f64;
        let i = (bytes_f.log(base).floor() as usize).min(units.len() - 1);
        if i == 0 {
            format!("{} {}", bytes, units[i])
        } else {
            let value = bytes_f / base.powi(i as i32);
            format!("{value:.2} {}", units[i])
        }
    }
}

// ──────────────────────────────────────────────────────────
//  TreeOption — 輸出選項
// ──────────────────────────────────────────────────────────

/// 樹狀輸出選項
#[derive(Debug, Clone)]
pub struct TreeOption {
    /// 大小格式（None = 不顯示）
    pub size: Option<SizeFormat>,
    /// 是否顯示最後修改時間
    pub last_modified: bool,
    /// 是否顯示建立時間
    pub created_at: bool,
}

impl Default for TreeOption {
    fn default() -> Self {
        Self {
            size: None,
            last_modified: false,
            created_at: false,
        }
    }
}

// ──────────────────────────────────────────────────────────
//  輸出函數
// ──────────────────────────────────────────────────────────

/// 將目錄的樹狀結構寫入 `writer`。
pub fn write_tree<W: IoWrite>(writer: &mut W, path: &Path, option: &TreeOption) -> io::Result<()> {
    let root_node = build_file_node(path)?;
    let root_info = format_node_info(&root_node, option);

    writeln!(writer, "# ******************************")?;
    writeln!(writer, "# Generation Time: {}", Local::now().format("%Y-%m-%d %H:%M:%S %z"))?;
    writeln!(writer, "# ******************************")?;
    writeln!(writer)?;
    writeln!(writer, "{}{}", root_node.name, root_info)?;

    if let Some(children) = root_node.children {
        write_tree_recursive(writer, &children, "", option)?;
    }

    writer.flush()?;
    Ok(())
}

/// 格式化節點資訊
fn format_node_info(node: &FileNode, option: &TreeOption) -> String {
    let mut info = String::new();

    if let Some(size_fmt) = &option.size {
        let _ = write!(info, " ({})", size_fmt.format(node.size));
    }
    if option.created_at {
        if let Some(t) = node.created_at {
            let _ = write!(info, " [cr: {}]", format_unix_to_local(t));
        }
    }
    if option.last_modified {
        if let Some(t) = node.last_modified {
            let _ = write!(info, " [mod: {}]", format_unix_to_local(t));
        }
    }

    info
}

/// 遞迴建構樹狀文字
fn write_tree_recursive<W: IoWrite>(
    writer: &mut W,
    nodes: &[FileNode],
    prefix: &str,
    option: &TreeOption,
) -> io::Result<()> {
    let mut iter = nodes.iter().peekable();

    while let Some(node) = iter.next() {
        let is_last = iter.peek().is_none();
        let connector = if is_last { "└── " } else { "├── " };
        let node_info = format_node_info(node, option);

        writeln!(writer, "{prefix}{connector}{}{node_info}", node.name)?;

        if let Some(children) = &node.children {
            let new_prefix = if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };
            write_tree_recursive(writer, children, &new_prefix, option)?;
        }
    }

    Ok(())
}

/// 將 Unix timestamp 格式化為本地時間字串
pub fn format_unix_to_local(secs: i64) -> String {
    match Local.timestamp_opt(secs, 0) {
        chrono::LocalResult::Single(dt)
        | chrono::LocalResult::Ambiguous(dt, _) => {
            dt.format("%Y-%m-%d %H:%M:%S %Z").to_string()
        }
        chrono::LocalResult::None => "無效的時間戳".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_format_binary() {
        let fmt = SizeFormat::Binary;
        assert_eq!(fmt.format(0), "0 B");
        assert_eq!(fmt.format(1023), "1023 B");
        assert_eq!(fmt.format(1024), "1.00 KiB");
        assert_eq!(fmt.format(2048), "2.00 KiB");
        assert_eq!(fmt.format(1048576), "1.00 MiB");
    }

    #[test]
    fn test_size_format_decimal() {
        let fmt = SizeFormat::Decimal;
        assert_eq!(fmt.format(1000), "1.00 KB");
        assert_eq!(fmt.format(1000000), "1.00 MB");
    }

    #[test]
    fn test_size_format_raw() {
        let fmt = SizeFormat::Raw;
        assert_eq!(fmt.format(0), "0 B");
        assert_eq!(fmt.format(1234), "1,234 B");
        assert_eq!(fmt.format(1234567), "1,234,567 B");
    }
}
