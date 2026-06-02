// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Hash 驗證檔與 JSON 結構檔的寫入器。
//!
//! # 安全準則
//!
//! - `parse_hash_file` 有行數上限（10M）防止 DoS
//! - `HashEntry` 建構時驗證 hex 格式與路徑基本合法性
//! - 路徑分割使用確定性邏輯（`find(' ')`），不依賴模糊的 `contains(" *")`

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use chrono::Local;

use crate::algorithm::HashData;
use crate::error::{OneeError, Result};
use super::node::FileNodeContainer;

/// 最大 hash 檔解析行數（DoS 防護）
const MAX_HASH_LINES: usize = 10_000_000;

// ──────────────────────────────────────────────────────────
//  HashEntry — 安全解析結果
// ──────────────────────────────────────────────────────────

/// 單筆 hash 驗證檔的解析結果。
///
/// 建構時會驗證：
/// - `hex` 為有效的十六進位字串（僅 `0-9a-fA-F`）
/// - `rel_path` 不為空
#[derive(Debug, Clone, PartialEq)]
pub struct HashEntry {
    /// 預期 hash hex 字串
    pub hash_hex: String,
    /// 相對路徑（未經正規化，保留原樣供後續 `sanitize_rel_path` 處理）
    pub rel_path: String,
}

impl HashEntry {
    /// 建立新的 `HashEntry`，執行基本驗證。
    ///
    /// ## 錯誤
    /// - hash 不是有效的 hex 字串
    /// - rel_path 為空
    pub fn new(hash_hex: &str, rel_path: &str) -> Result<Self> {
        let hash = hash_hex.trim().to_string();
        let path = rel_path.trim().to_string();

        if hash.is_empty() {
            return Err(OneeError::HashFileParseError {
                line: 0,
                detail: "hash 值為空".into(),
            });
        }

        if !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            // 允許大小寫 a-f
            return Err(OneeError::HashFileParseError {
                line: 0,
                detail: format!("hash 包含非十六進位字元: {hash}"),
            });
        }

        if path.is_empty() {
            return Err(OneeError::HashFileParseError {
                line: 0,
                detail: "路徑為空".into(),
            });
        }

        // 拒絕路徑中的 null byte（底層 C 截斷攻擊）
        if path.contains('\0') {
            return Err(OneeError::HashFileParseError {
                line: 0,
                detail: "路徑包含 null byte".into(),
            });
        }

        Ok(Self {
            hash_hex: hash,
            rel_path: path,
        })
    }

    /// 回傳該 hash 的預期 byte 長度（hex length / 2）
    pub fn hash_byte_length(&self) -> usize {
        self.hash_hex.len() / 2
    }
}

// ──────────────────────────────────────────────────────────
//  寫入器
// ──────────────────────────────────────────────────────────

/// 將 hash 結果寫入驗證檔（`shasum` 相容格式）。
///
/// 格式：
/// ```text
/// <hash_hex> *<relative_path>
/// ```
///
/// `*` 表示二進位模式。
pub fn save_hash_file(
    data: &[HashData],
    output_path: &Path,
    root_path: &Path,
) -> io::Result<()> {
    let mut writer = BufWriter::new(File::create(output_path)?);

    writeln!(writer, "# ******************************")?;
    writeln!(writer, "# Total Files Count: {}", data.len())?;
    writeln!(writer, "# Generation Time: {}", Local::now().format("%Y-%m-%d %H:%M:%S %z"))?;
    writeln!(writer, "# ******************************")?;

    let mut hex_buf = String::with_capacity(128); // SHA-512 hex = 128 chars
    for entry in data {
        let rel_path = entry
            .path
            .strip_prefix(root_path)
            .unwrap_or(&entry.path);
        let path_str = rel_path.to_string_lossy().replace('\\', "/");
        hex_buf.clear();
        entry.hash_hex_into(&mut hex_buf);
        writeln!(writer, "{hex_buf} *{path_str}")?;
    }

    writer.flush()?;
    Ok(())
}

/// 將 `FileNodeContainer` 寫入 JSON 檔案（含縮排）。
pub fn save_file_node_json(
    container: &FileNodeContainer,
    output_path: &Path,
) -> io::Result<()> {
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, container)?;
    Ok(())
}

// ──────────────────────────────────────────────────────────
//  解析器
// ──────────────────────────────────────────────────────────

/// 解析 hash 驗證檔，回傳 `HashEntry` 列表。
///
/// # 格式支援
///
/// 相容於 GNU/BSD `sha*sum` 格式：
/// ```text
/// <hash_hex> *<path>   # 二進位模式（binary mode）
/// <hash_hex>  <path>   # 文字模式（text mode，兩個空格）
/// ```
///
/// # 安全防護
///
/// - 行數上限 10,000,000 行（DoS 防護）
/// - 每筆記錄執行 hex 格式驗證
/// - 跳過註解行（`#`）與空白行
///
/// # 確定性分割演算法
///
/// 1. 找第一個空格位置 → hash 結束
/// 2. 跳過空格後的模式字元（`*` 或空格）→ path 開始
/// 3. 剩餘部分即為路徑（保留空格）
pub fn parse_hash_file(content: &str) -> Vec<HashEntry> {
    content
        .lines()
        .enumerate()
        .take(MAX_HASH_LINES)
        .filter(|(_, line)| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .filter_map(|(lineno, line)| {
            let line = line.trim();

            // 找第一個空格 — hash 到這裡結束
            let space_pos = line.find(' ')?;
            let hash_part = &line[..space_pos];
            let rest = &line[space_pos + 1..];

            // 跳過模式字元（binary mode 用 *，text mode 用空格或不跳）
            let path_part = if let Some(p) = rest.strip_prefix('*') {
                p
            } else if let Some(p) = rest.strip_prefix(' ') {
                // text mode：hash + 兩個空格 + path
                p
            } else {
                // 無模式字元也接受
                rest
            };

            // 用 HashEntry::new 驗證
            match HashEntry::new(hash_part, path_part) {
                Ok(entry) => Some(entry),
                Err(e) => {
                    // 解析錯誤時，在錯誤訊息中註記行號
                    eprintln!(
                        "⚠ 跳過第 {} 行: {}",
                        lineno + 1,
                        e
                    );
                    None
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_entry_validation() {
        assert!(HashEntry::new("abc123", "file.txt").is_ok());
        assert!(HashEntry::new("", "file.txt").is_err());
        assert!(HashEntry::new("abc123", "").is_err());
        assert!(HashEntry::new("xyz!!!", "file.txt").is_err());
        assert!(HashEntry::new("abc123", "file\0.txt").is_err());
    }

    #[test]
    fn test_parse_hash_file_standard() {
        let content = "# comment
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 *file1.txt
d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592  file2.txt

# another comment
789abc *sub/file3.bin";
        let entries = parse_hash_file(content);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].hash_hex, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(entries[0].rel_path, "file1.txt");
        assert_eq!(entries[1].rel_path, "file2.txt");
        assert_eq!(entries[2].rel_path, "sub/file3.bin");
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse_hash_file("").is_empty());
        assert!(parse_hash_file("# only comments").is_empty());
    }

    #[test]
    fn test_parse_malformed_lines() {
        // 無效的 hex 應該被跳過（不 panic）
        let content = "zzzz *file.txt\nabc123 *ok.txt";
        let entries = parse_hash_file(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].rel_path, "ok.txt");
    }

    #[test]
    fn test_hash_entry_rejects_null_byte() {
        assert!(HashEntry::new("abc", "bad\0file.txt").is_err());
    }

    #[test]
    fn test_hash_entry_rejects_non_hex() {
        assert!(HashEntry::new("not-hex-!!", "f.txt").is_err());
        // 大小寫都接受
        assert!(HashEntry::new("ABCDEFabcdef", "f.txt").is_ok());
    }
}
