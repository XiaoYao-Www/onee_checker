// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! manifest 解析與比對模組。
//!
//! 支援多種校驗檔格式（Shasum、BSD、CSV、JSON）的解析與自動偵測，
//! 並提供兩份 manifest 的差異比對（`DiffReport`）。

use std::collections::BTreeMap;

use crate::error::{OneeError, Result};
use crate::fs::HashEntry;

/// 支援的 Manifest 格式類型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManifestFormat {
    /// Shasum 格式 (`<hex>  <path>` 或 `<hex> *<path>`)
    Shasum,
    /// BSD 格式 (`ALGORITHM (path) = hex`)
    Bsd,
    /// CSV 格式 (`path,algorithm,hex` 或帶表頭)
    Csv,
    /// JSON 格式 (陣列或含 `entries` 欄位的物件)
    Json,
}

impl ManifestFormat {
    /// 回傳格式的字串名稱。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shasum => "Shasum",
            Self::Bsd => "Bsd",
            Self::Csv => "Csv",
            Self::Json => "Json",
        }
    }

    /// 根據副檔名判定 Manifest 格式。
    ///
    /// - sha1 / sha256 / sha512 / sha3* / md5 / blake2* / blake3 → Shasum
    /// - csv → Csv
    /// - json → Json
    /// - 其他 → None
    #[must_use]
    pub fn from_suffix(suffix: &str) -> Option<Self> {
        let lower = suffix.to_lowercase();
        let trimmed = lower.trim_start_matches('.');
        match trimmed {
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            s if s.starts_with("sha1")
                || s.starts_with("sha256")
                || s.starts_with("sha512")
                || s.starts_with("sha3")
                || s.starts_with("md5")
                || s.starts_with("blake2")
                || s.starts_with("blake3")
                || s.starts_with("shasum")
                || s.starts_with("sum") =>
            {
                Some(Self::Shasum)
            }
            _ => None,
        }
    }
}

/// 依內容特徵判斷 Manifest 格式，優先序 Bsd > Json > Csv > Shasum。
#[must_use]
pub fn detect_format(content: &str) -> Option<ManifestFormat> {
    // 1. 檢查 BSD 格式特徵：如 `SHA256 (...) = <hex>` 或 `MD5 (...) = <hex>`
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(open_idx) = trimmed.find('(') {
            if let Some(close_idx) = trimmed.find(')') {
                if open_idx < close_idx && trimmed[close_idx..].contains('=') {
                    return Some(ManifestFormat::Bsd);
                }
            }
        }
    }

    // 2. 檢查 JSON 格式：開頭為 `{` 或 `[`（忽略空白與 BOM）
    let cleaned = content.trim_start_matches('\u{feff}').trim_start();
    if cleaned.starts_with('{') || cleaned.starts_with('[') {
        return Some(ManifestFormat::Json);
    }

    // 3. 檢查 CSV 格式：含逗號，且非註解行含有逗號
    let mut has_comma = false;
    let mut line_count = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        line_count += 1;
        if trimmed.contains(',') {
            has_comma = true;
        }
    }
    if has_comma && line_count > 0 {
        return Some(ManifestFormat::Csv);
    }

    // 4. Fallback / Shasum 檢查
    if line_count > 0 {
        return Some(ManifestFormat::Shasum);
    }

    None
}

/// 自動偵測格式並解析 manifest 內容。
pub fn parse_auto(content: &str) -> Result<(ManifestFormat, Vec<HashEntry>)> {
    if let Some(format) = detect_format(content) {
        match parse(content, format) {
            Ok(entries) => return Ok((format, entries)),
            Err(_) => {
                // 若偵測到的格式解析失敗，嘗試 Shasum fallback
                if format != ManifestFormat::Shasum {
                    if let Ok(entries) = parse(content, ManifestFormat::Shasum) {
                        return Ok((ManifestFormat::Shasum, entries));
                    }
                }
            }
        }
    }

    // 最終 fallback 到 Shasum
    let entries = parse(content, ManifestFormat::Shasum)?;
    Ok((ManifestFormat::Shasum, entries))
}

/// 依指定格式解析 manifest 內容。
pub fn parse(content: &str, format: ManifestFormat) -> Result<Vec<HashEntry>> {
    // 移除 UTF-8 BOM
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);

    match format {
        ManifestFormat::Shasum => parse_shasum(content),
        ManifestFormat::Bsd => parse_bsd(content),
        ManifestFormat::Csv => parse_csv(content),
        ManifestFormat::Json => parse_json(content),
    }
}

fn parse_shasum(content: &str) -> Result<Vec<HashEntry>> {
    let mut entries = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some(space_pos) = line.find(' ') else {
            return Err(OneeError::HashFileParseError {
                line: line_num,
                detail: "缺少 hash 與路徑之間的分隔空格".into(),
            });
        };
        let hash_part = &line[..space_pos];
        let rest = &line[space_pos + 1..];
        let path_part = if let Some(p) = rest.strip_prefix('*') {
            p
        } else if let Some(p) = rest.strip_prefix(' ') {
            p
        } else {
            rest
        };

        let entry = HashEntry::new(hash_part, path_part).map_err(|e| match e {
            OneeError::HashFileParseError { detail, .. } => {
                OneeError::HashFileParseError { line: line_num, detail }
            }
            _ => OneeError::HashFileParseError { line: line_num, detail: e.to_string() },
        })?;
        entries.push(entry);
    }

    if entries.is_empty() {
        return Err(OneeError::HashFileParseError {
            line: 0,
            detail: "沒有可驗證的 hash 記錄".into(),
        });
    }
    Ok(entries)
}

fn parse_bsd(content: &str) -> Result<Vec<HashEntry>> {
    let mut entries = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 格式: ALGORITHM (path) = hex
        let Some(open_paren) = line.find('(') else {
            return Err(OneeError::HashFileParseError {
                line: line_num,
                detail: "BSD 格式缺少 '('".into(),
            });
        };
        let Some(close_paren) = line.rfind(')') else {
            return Err(OneeError::HashFileParseError {
                line: line_num,
                detail: "BSD 格式缺少 ')'".into(),
            });
        };
        if open_paren >= close_paren {
            return Err(OneeError::HashFileParseError {
                line: line_num,
                detail: "BSD 格式括號位置無效".into(),
            });
        }

        let path_part = &line[open_paren + 1..close_paren];
        let after_paren = &line[close_paren + 1..];
        let Some(eq_pos) = after_paren.find('=') else {
            return Err(OneeError::HashFileParseError {
                line: line_num,
                detail: "BSD 格式缺少 '='".into(),
            });
        };
        let hash_part = after_paren[eq_pos + 1..].trim();

        let entry = HashEntry::new(hash_part, path_part).map_err(|e| match e {
            OneeError::HashFileParseError { detail, .. } => {
                OneeError::HashFileParseError { line: line_num, detail }
            }
            _ => OneeError::HashFileParseError { line: line_num, detail: e.to_string() },
        })?;
        entries.push(entry);
    }

    if entries.is_empty() {
        return Err(OneeError::HashFileParseError {
            line: 0,
            detail: "沒有可驗證的 hash 記錄".into(),
        });
    }
    Ok(entries)
}

fn parse_csv(content: &str) -> Result<Vec<HashEntry>> {
    let mut entries = Vec::new();
    let mut is_first_line = true;

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fields = parse_csv_line(line)
            .map_err(|d| OneeError::HashFileParseError { line: line_num, detail: d })?;

        if fields.is_empty() {
            continue;
        }

        if is_first_line {
            is_first_line = false;
            let lower_fields: Vec<String> = fields.iter().map(|f| f.to_lowercase()).collect();
            if lower_fields
                .iter()
                .any(|f| f.contains("path") || f.contains("hash") || f.contains("algorithm"))
            {
                continue;
            }
        }

        let (path_str, hash_str) = if fields.len() >= 3 {
            let f0 = &fields[0];
            let f1 = &fields[1];
            let f2 = &fields[2];
            if f2.bytes().all(|b| b.is_ascii_hexdigit()) && !f2.is_empty() {
                (f0.as_str(), f2.as_str())
            } else if f1.bytes().all(|b| b.is_ascii_hexdigit()) && !f1.is_empty() {
                (f0.as_str(), f1.as_str())
            } else {
                (f0.as_str(), f2.as_str())
            }
        } else if fields.len() == 2 {
            (fields[0].as_str(), fields[1].as_str())
        } else {
            return Err(OneeError::HashFileParseError {
                line: line_num,
                detail: format!("CSV 欄位不足: {}", fields.len()),
            });
        };

        let entry = HashEntry::new(hash_str, path_str).map_err(|e| match e {
            OneeError::HashFileParseError { detail, .. } => {
                OneeError::HashFileParseError { line: line_num, detail }
            }
            _ => OneeError::HashFileParseError { line: line_num, detail: e.to_string() },
        })?;
        entries.push(entry);
    }

    if entries.is_empty() {
        return Err(OneeError::HashFileParseError {
            line: 0,
            detail: "沒有可驗證的 hash 記錄".into(),
        });
    }
    Ok(entries)
}

/// 簡單且健全的 CSV 單行解析器（支援引號與雙引號轉義 `""`）。
fn parse_csv_line(line: &str) -> std::result::Result<Vec<String>, String> {
    let mut fields = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current_field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                current_field.push(c);
            }
        } else {
            if c == '"' {
                in_quotes = true;
            } else if c == ',' {
                fields.push(current_field);
                current_field = String::new();
            } else {
                current_field.push(c);
            }
        }
    }

    if in_quotes {
        return Err("CSV 引號未關閉".into());
    }

    fields.push(current_field);
    Ok(fields)
}

fn parse_json(content: &str) -> Result<Vec<HashEntry>> {
    let v: serde_json::Value = serde_json::from_str(content)?;

    // 檢查是否為 oneechk 目錄結構 JSON（含 nodes 欄位）
    if let Some(obj) = v.as_object() {
        if obj.contains_key("nodes")
            || (obj.contains_key("root_path") && !obj.contains_key("entries"))
        {
            return Err(OneeError::HashFileParseError {
                line: 0,
                detail: "oneechk json 產出的目錄結構 JSON（含 nodes）不是 manifest".into(),
            });
        }
    }

    let items = if let Some(arr) = v.as_array() {
        arr.clone()
    } else if let Some(obj) = v.as_object() {
        if let Some(entries_val) = obj.get("entries") {
            if let Some(arr) = entries_val.as_array() {
                arr.clone()
            } else {
                return Err(OneeError::HashFileParseError {
                    line: 0,
                    detail: "JSON 'entries' 欄位不是陣列".into(),
                });
            }
        } else {
            return Err(OneeError::HashFileParseError {
                line: 0,
                detail: "JSON 物件缺少 'entries' 欄位且非頂層陣列".into(),
            });
        }
    } else {
        return Err(OneeError::HashFileParseError {
            line: 0,
            detail: "JSON 格式必須是陣列或包含 entries 的物件".into(),
        });
    };

    let mut entries = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        let line_num = idx + 1;
        let path = item
            .get("path")
            .or_else(|| item.get("rel_path"))
            .and_then(|val| val.as_str())
            .ok_or_else(|| OneeError::HashFileParseError {
            line: line_num,
            detail: "JSON 項目缺少 path 或 rel_path 欄位".into(),
        })?;

        let hash = item
            .get("hash")
            .or_else(|| item.get("hash_hex"))
            .or_else(|| item.get("hex"))
            .and_then(|val| val.as_str())
            .ok_or_else(|| OneeError::HashFileParseError {
                line: line_num,
                detail: "JSON 項目缺少 hash、hash_hex 或 hex 欄位".into(),
            })?;

        let entry = HashEntry::new(hash, path).map_err(|e| match e {
            OneeError::HashFileParseError { detail, .. } => {
                OneeError::HashFileParseError { line: line_num, detail }
            }
            _ => OneeError::HashFileParseError { line: line_num, detail: e.to_string() },
        })?;
        entries.push(entry);
    }

    if entries.is_empty() {
        return Err(OneeError::HashFileParseError {
            line: 0,
            detail: "JSON manifest 沒有任何 hash 記錄".into(),
        });
    }

    Ok(entries)
}

// ──────────────────────────────────────────────────────────
//  Diff 模組
// ──────────────────────────────────────────────────────────

/// 變更的 Hash 記錄。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ChangedEntry {
    /// 檔案相對路徑
    pub path: String,
    /// 舊 hash 值
    pub old_hash: String,
    /// 新 hash 值
    pub new_hash: String,
}

/// 兩份 Manifest 差異比對報告。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct DiffReport {
    /// 新增的檔案路徑列表
    pub added: Vec<String>,
    /// 移除的檔案路徑列表
    pub removed: Vec<String>,
    /// 變更的檔案列表（含舊新 hash）
    pub changed: Vec<ChangedEntry>,
    /// 未變更的檔案數量
    pub unchanged: u64,
}

impl DiffReport {
    /// 檢查報告是否完全沒有差異（added, removed, changed 皆為空）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// 回傳總變更數量（added + removed + changed 的總數）。
    #[must_use]
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.removed.len() + self.changed.len()
    }
}

/// 比對兩份 `HashEntry` 切片，產出穩定排序的 `DiffReport`。
///
/// - 以 `rel_path` 為鍵
/// - hash 比較大小寫不敏感（`eq_ignore_ascii_case`）
/// - 結果依 path 升冪排序
#[must_use]
pub fn diff(old: &[HashEntry], new: &[HashEntry]) -> DiffReport {
    let mut old_map = BTreeMap::new();
    for entry in old {
        old_map.insert(entry.rel_path.clone(), entry.hash_hex.clone());
    }

    let mut new_map = BTreeMap::new();
    for entry in new {
        new_map.insert(entry.rel_path.clone(), entry.hash_hex.clone());
    }

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    let mut unchanged = 0;

    for (path, new_hash) in &new_map {
        if let Some(old_hash) = old_map.get(path) {
            if old_hash.eq_ignore_ascii_case(new_hash) {
                unchanged += 1;
            } else {
                changed.push(ChangedEntry {
                    path: path.clone(),
                    old_hash: old_hash.clone(),
                    new_hash: new_hash.clone(),
                });
            }
        } else {
            added.push(path.clone());
        }
    }

    for path in old_map.keys() {
        if !new_map.contains_key(path) {
            removed.push(path.clone());
        }
    }

    added.sort();
    removed.sort();
    changed.sort_by(|a, b| a.path.cmp(&b.path));

    DiffReport { added, removed, changed, unchanged }
}

// ──────────────────────────────────────────────────────────
//  單元測試
// ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_as_str_and_suffix() {
        assert_eq!(ManifestFormat::Shasum.as_str(), "Shasum");
        assert_eq!(ManifestFormat::Bsd.as_str(), "Bsd");
        assert_eq!(ManifestFormat::Csv.as_str(), "Csv");
        assert_eq!(ManifestFormat::Json.as_str(), "Json");

        assert_eq!(ManifestFormat::from_suffix("sha256"), Some(ManifestFormat::Shasum));
        assert_eq!(ManifestFormat::from_suffix(".sha1"), Some(ManifestFormat::Shasum));
        assert_eq!(ManifestFormat::from_suffix("md5"), Some(ManifestFormat::Shasum));
        assert_eq!(ManifestFormat::from_suffix("blake3"), Some(ManifestFormat::Shasum));
        assert_eq!(ManifestFormat::from_suffix("csv"), Some(ManifestFormat::Csv));
        assert_eq!(ManifestFormat::from_suffix(".JSON"), Some(ManifestFormat::Json));
        assert_eq!(ManifestFormat::from_suffix("txt"), None);
    }

    #[test]
    fn test_detect_format() {
        let bsd_content =
            "SHA256 (file1.txt) = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(detect_format(bsd_content), Some(ManifestFormat::Bsd));

        let json_content = "[{\"path\": \"a.txt\", \"hash\": \"abc\"}]";
        assert_eq!(detect_format(json_content), Some(ManifestFormat::Json));

        let csv_content = "path,hash\nfile1.txt,abc123";
        assert_eq!(detect_format(csv_content), Some(ManifestFormat::Csv));

        let shasum_content =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 *file1.txt";
        assert_eq!(detect_format(shasum_content), Some(ManifestFormat::Shasum));
    }

    #[test]
    fn test_parse_shasum() {
        let content = "\u{feff}# BOM and comment\n\
                       e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 *file1.txt\r\n\
                       d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592  file2.txt";
        let entries = parse(content, ManifestFormat::Shasum).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].rel_path, "file1.txt");
        assert_eq!(entries[1].rel_path, "file2.txt");
    }

    #[test]
    fn test_parse_bsd() {
        let content = "SHA256 (file1.txt) = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n\
                       MD5 (dir/file2.bin) = d41d8cd98f00b204e9800998ecf8427e";
        let entries = parse(content, ManifestFormat::Bsd).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].rel_path, "file1.txt");
        assert_eq!(entries[1].rel_path, "dir/file2.bin");
        assert_eq!(entries[1].hash_hex, "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn test_parse_csv_with_quotes_and_header() {
        let content = "path,algorithm,hash\n\
                       \"file with, comma.txt\",sha256,e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n\
                       \"file with \"\"quotes\"\".txt\",sha256,d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592";
        let entries = parse(content, ManifestFormat::Csv).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].rel_path, "file with, comma.txt");
        assert_eq!(entries[1].rel_path, "file with \"quotes\".txt");
    }

    #[test]
    fn test_parse_json_variants() {
        let arr_json = "[{\"path\": \"a.txt\", \"hash\": \"abc123\"}]";
        let entries = parse(arr_json, ManifestFormat::Json).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].rel_path, "a.txt");

        let obj_json = "{\"entries\": [{\"rel_path\": \"b.txt\", \"hash_hex\": \"def456\"}]}";
        let entries2 = parse(obj_json, ManifestFormat::Json).unwrap();
        assert_eq!(entries2.len(), 1);
        assert_eq!(entries2[0].rel_path, "b.txt");
        assert_eq!(entries2[0].hash_hex, "def456");

        let tree_json = "{\"root_path\": \".\", \"nodes\": []}";
        let res = parse(tree_json, ManifestFormat::Json);
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_error_line_number() {
        let content =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 *file1.txt\n\
                       invalid_hash_line_2 *file2.txt";
        let res = parse(content, ManifestFormat::Shasum);
        assert!(res.is_err());
        if let Err(OneeError::HashFileParseError { line, .. }) = res {
            assert_eq!(line, 2);
        } else {
            panic!("Expected HashFileParseError");
        }
    }

    #[test]
    fn test_parse_auto_fallback() {
        let content = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 *file1.txt";
        let (fmt, entries) = parse_auto(content).unwrap();
        assert_eq!(fmt, ManifestFormat::Shasum);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_diff_report() {
        let old = vec![
            HashEntry::new("aaa", "unchanged.txt").unwrap(),
            HashEntry::new("bbb", "changed.txt").unwrap(),
            HashEntry::new("ccc", "removed.txt").unwrap(),
        ];
        let new = vec![
            HashEntry::new("AAA", "unchanged.txt").unwrap(),
            HashEntry::new("ddd", "changed.txt").unwrap(),
            HashEntry::new("eee", "added.txt").unwrap(),
        ];

        let report = diff(&old, &new);
        assert_eq!(report.unchanged, 1);
        assert_eq!(report.added, vec!["added.txt".to_string()]);
        assert_eq!(report.removed, vec!["removed.txt".to_string()]);
        assert_eq!(report.changed.len(), 1);
        assert_eq!(report.changed[0].path, "changed.txt");
        assert_eq!(report.changed[0].old_hash, "bbb");
        assert_eq!(report.changed[0].new_hash, "ddd");

        assert!(!report.is_empty());
        assert_eq!(report.total_changes(), 3);
    }
}
