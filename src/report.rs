// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! 機器可解析輸出層模組 (`report`)。
//!
//! 提供結構化報告（JSON、NDJSON、CSV、Shasum、BSD、Human 等輸出格式）、
//! 驗證摘要計算、狀態列舉以及各種格式化的寫入函式。

use serde::Serialize;
use std::io::{self, Write};

use crate::algorithm::HashMode;

/// 輸出格式列舉。
#[derive(clap::ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[clap(rename_all = "lower")]
pub enum OutputFormat {
    /// 人類可讀格式（預設）
    #[default]
    Human,
    /// 傳統 Shasum 格式
    Shasum,
    /// BSD 格式
    Bsd,
    /// JSON 格式
    Json,
    /// CSV 格式
    Csv,
    /// NDJSON 格式
    Ndjson,
}

/// 項目驗證狀態列舉。
#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntryStatus {
    /// 驗證相符
    Match,
    /// 驗證不相符
    Mismatch,
    /// 發生錯誤
    Error,
    /// 檔案遺失
    Missing,
}

impl EntryStatus {
    /// 取得狀態的小寫字串表示。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Mismatch => "mismatch",
            Self::Error => "error",
            Self::Missing => "missing",
        }
    }
}

/// 單一項目的驗證報告。
#[derive(Serialize, Clone, Debug)]
pub struct EntryReport {
    /// 檔案相對路徑
    pub path: String,
    /// 驗證狀態
    pub status: EntryStatus,
    /// 預期雜湊值
    pub expected: Option<String>,
    /// 實際雜湊值
    pub actual: Option<String>,
    /// 錯誤訊息（None 時序列化為 null）
    pub error: Option<String>,
}

impl EntryReport {
    /// 建立新的 `EntryReport` 實例，預設 expected、actual、error 皆為 None。
    #[must_use]
    pub fn new(path: impl Into<String>, status: EntryStatus) -> Self {
        Self { path: path.into(), status, expected: None, actual: None, error: None }
    }
}

/// 驗證摘要統計。
#[derive(Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VerifySummary {
    /// 總計項目數
    pub total: u64,
    /// 相符數量
    pub matched: u64,
    /// 不相符數量
    pub mismatched: u64,
    /// 錯誤數量
    pub errors: u64,
    /// 遺失數量
    pub missing: u64,
}

impl VerifySummary {
    /// 根據傳入的狀態更新摘要計數。
    pub fn add(&mut self, status: EntryStatus) {
        self.total += 1;
        match status {
            EntryStatus::Match => self.matched += 1,
            EntryStatus::Mismatch => self.mismatched += 1,
            EntryStatus::Error => self.errors += 1,
            EntryStatus::Missing => self.missing += 1,
        }
    }

    /// 從項目報告切片建立摘要。
    #[must_use]
    pub fn from_entries(entries: &[EntryReport]) -> Self {
        let mut summary = Self::default();
        for entry in entries {
            summary.add(entry.status);
        }
        summary
    }
}

/// 整體驗證報告。
#[derive(Serialize, Clone, Debug)]
pub struct VerifyReport {
    /// 程式版本
    pub version: String,
    /// 執行的命令
    pub command: String,
    /// 雜湊檔案路徑
    pub hashfile: String,
    /// 演算法名稱
    pub algorithm: String,
    /// 根目錄
    pub root: String,
    /// 驗證摘要
    pub summary: VerifySummary,
    /// 離開代碼
    pub exit_code: i32,
    /// 所有項目報告列表
    pub entries: Vec<EntryReport>,
}

impl VerifyReport {
    /// 計算離開代碼：errors > 0 → 2；mismatched > 0 → 1；total == 0 → 4；否則 0。
    #[must_use]
    pub const fn compute_exit_code(&self) -> i32 {
        if self.summary.errors > 0 {
            2
        } else if self.summary.mismatched > 0 {
            1
        } else if self.summary.total == 0 {
            4
        } else {
            0
        }
    }
}

/// 將欄位進行 CSV 轉義：含逗號、雙引號、CR/LF 時以雙引號包裹並將 `"` 轉為 `""`。
#[must_use]
pub fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\r') || field.contains('\n') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        field.to_string()
    }
}

/// 將單筆雜湊結果寫入輸出串流。
///
/// `mode` 只影響 shasum 系列格式（文字模式兩個空白、二進位模式 ` *`）；
/// JSON/NDJSON 會額外帶出 `mode` 欄位，CSV 與 BSD 格式不受影響。
pub fn write_hash_line<W: Write>(
    w: &mut W,
    format: OutputFormat,
    algorithm: &str,
    hash_hex: &str,
    rel_path: &str,
    mode: HashMode,
) -> io::Result<()> {
    let normalized_path = rel_path.replace('\\', "/");
    match format {
        OutputFormat::Shasum | OutputFormat::Human => {
            writeln!(w, "{hash_hex}{}{normalized_path}", mode.separator())
        }
        OutputFormat::Bsd => {
            let algo_lower = algorithm.to_lowercase();
            writeln!(w, "{algo_lower} ({normalized_path}) = {hash_hex}")
        }
        OutputFormat::Csv => {
            let escaped = csv_escape(&normalized_path);
            writeln!(w, "{hash_hex},{escaped}")
        }
        OutputFormat::Json | OutputFormat::Ndjson => {
            let algo_lower = algorithm.to_lowercase();
            let json_obj = serde_json::json!({
                "algorithm": algo_lower,
                "hash": hash_hex,
                "mode": mode.as_str(),
                "path": normalized_path,
            });
            writeln!(w, "{json_obj}")
        }
    }
}

/// 將單筆驗證項目寫入輸出串流。
pub fn write_entry<W: Write>(
    w: &mut W,
    format: OutputFormat,
    entry: &EntryReport,
    mode: HashMode,
) -> io::Result<()> {
    let normalized_path = entry.path.replace('\\', "/");
    match format {
        OutputFormat::Human | OutputFormat::Json => Ok(()),
        OutputFormat::Ndjson => {
            let json_str = serde_json::to_string(entry)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            writeln!(w, "{json_str}")
        }
        OutputFormat::Csv => {
            let p_esc = csv_escape(&normalized_path);
            let s_esc = csv_escape(entry.status.as_str());
            let exp = entry.expected.as_deref().unwrap_or("");
            let exp_esc = csv_escape(exp);
            let act = entry.actual.as_deref().unwrap_or("");
            let act_esc = csv_escape(act);
            writeln!(w, "{p_esc},{s_esc},{exp_esc},{act_esc}")
        }
        OutputFormat::Shasum | OutputFormat::Bsd => {
            if let Some(actual) = &entry.actual {
                write_hash_line(w, format, "", actual, &normalized_path, mode)
            } else {
                Ok(())
            }
        }
    }
}

/// 將完整驗證報告寫入輸出串流。
///
/// `Shasum`／`Bsd` 會把每筆「實際 hash」重新輸出成清單（可用於正規化既有清單）；
/// `mode` 決定 shasum 的分隔符。
pub fn write_verify_report<W: Write>(
    w: &mut W,
    format: OutputFormat,
    report: &VerifyReport,
    mode: HashMode,
) -> io::Result<()> {
    match format {
        OutputFormat::Json => {
            let json_str = serde_json::to_string_pretty(report)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            writeln!(w, "{json_str}")
        }
        OutputFormat::Ndjson => {
            for entry in &report.entries {
                let json_str = serde_json::to_string(entry)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                writeln!(w, "{json_str}")?;
            }
            let summary_obj = serde_json::json!({
                "summary": report.summary,
            });
            writeln!(w, "{summary_obj}")
        }
        OutputFormat::Csv => {
            writeln!(w, "path,status,expected,actual")?;
            for entry in &report.entries {
                let normalized_path = entry.path.replace('\\', "/");
                let p_esc = csv_escape(&normalized_path);
                let s_esc = csv_escape(entry.status.as_str());
                let exp = entry.expected.as_deref().unwrap_or("");
                let exp_esc = csv_escape(exp);
                let act = entry.actual.as_deref().unwrap_or("");
                let act_esc = csv_escape(act);
                writeln!(w, "{p_esc},{s_esc},{exp_esc},{act_esc}")?;
            }
            Ok(())
        }
        OutputFormat::Human => Ok(()),
        OutputFormat::Shasum | OutputFormat::Bsd => {
            for entry in &report.entries {
                if let Some(actual) = &entry.actual {
                    write_hash_line(w, format, &report.algorithm, actual, &entry.path, mode)?;
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_status_serialization() {
        assert_eq!(serde_json::to_string(&EntryStatus::Match).unwrap(), "\"match\"");
        assert_eq!(serde_json::to_string(&EntryStatus::Mismatch).unwrap(), "\"mismatch\"");
        assert_eq!(serde_json::to_string(&EntryStatus::Error).unwrap(), "\"error\"");
        assert_eq!(serde_json::to_string(&EntryStatus::Missing).unwrap(), "\"missing\"");
        assert_eq!(EntryStatus::Match.as_str(), "match");
    }

    #[test]
    fn test_entry_report_json_error_null() {
        let entry = EntryReport::new("test.txt", EntryStatus::Match);
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"error\":null"));
    }

    #[test]
    fn test_compute_exit_code() {
        let mut r = VerifyReport {
            version: "2.1.0".to_string(),
            command: "verify".to_string(),
            hashfile: "sha256sums.txt".to_string(),
            algorithm: "sha256".to_string(),
            root: ".".to_string(),
            summary: VerifySummary::default(),
            exit_code: 0,
            entries: vec![],
        };
        assert_eq!(r.compute_exit_code(), 4);

        r.summary.total = 1;
        r.summary.matched = 1;
        assert_eq!(r.compute_exit_code(), 0);

        r.summary.mismatched = 1;
        assert_eq!(r.compute_exit_code(), 1);

        r.summary.errors = 1;
        assert_eq!(r.compute_exit_code(), 2);
    }

    #[test]
    fn test_write_hash_line_shasum_bsd() {
        let mut buf = Vec::new();
        write_hash_line(
            &mut buf,
            OutputFormat::Shasum,
            "SHA256",
            "abc12345",
            "foo\\bar.txt",
            HashMode::Text,
        )
        .unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(s, "abc12345  foo/bar.txt\n");

        let mut buf2 = Vec::new();
        write_hash_line(
            &mut buf2,
            OutputFormat::Bsd,
            "SHA256",
            "abc12345",
            "foo\\bar.txt",
            HashMode::Text,
        )
        .unwrap();
        let s2 = String::from_utf8(buf2).unwrap();
        assert_eq!(s2, "sha256 (foo/bar.txt) = abc12345\n");
    }

    /// 二進位模式的分隔符是 ` *`，文字模式是兩個空白。
    #[test]
    fn test_hash_mode_separator() {
        assert_eq!(HashMode::Text.separator(), "  ");
        assert_eq!(HashMode::Binary.separator(), " *");
        assert_eq!(HashMode::default(), HashMode::Text);
        assert_eq!(HashMode::Binary.as_str(), "binary");

        let mut buf = Vec::new();
        write_hash_line(
            &mut buf,
            OutputFormat::Shasum,
            "SHA256",
            "dead",
            "a.txt",
            HashMode::Binary,
        )
        .unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "dead *a.txt\n");

        let mut json = Vec::new();
        write_hash_line(&mut json, OutputFormat::Json, "sha256", "dead", "a.txt", HashMode::Binary)
            .unwrap();
        let value: serde_json::Value =
            serde_json::from_str(std::str::from_utf8(&json).unwrap().trim()).unwrap();
        assert_eq!(value["mode"], "binary");
    }

    /// `verify --format shasum` 應把實際 hash 重新輸出成清單，並套用模式字元。
    #[test]
    fn test_write_verify_report_shasum() {
        let report = VerifyReport {
            version: "2.2.0".to_string(),
            command: "verify".to_string(),
            hashfile: "sums.txt".to_string(),
            algorithm: "SHA-256".to_string(),
            root: ".".to_string(),
            summary: VerifySummary { total: 1, matched: 1, mismatched: 0, errors: 0, missing: 0 },
            exit_code: 0,
            entries: vec![EntryReport {
                path: "dir/a.txt".to_string(),
                status: EntryStatus::Match,
                expected: Some("abc".to_string()),
                actual: Some("abc".to_string()),
                error: None,
            }],
        };

        let mut buf = Vec::new();
        write_verify_report(&mut buf, OutputFormat::Shasum, &report, HashMode::Binary).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "abc *dir/a.txt\n");

        let mut buf2 = Vec::new();
        write_verify_report(&mut buf2, OutputFormat::Shasum, &report, HashMode::Text).unwrap();
        assert_eq!(String::from_utf8(buf2).unwrap(), "abc  dir/a.txt\n");
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("normal"), "normal");
        assert_eq!(csv_escape("foo,bar"), "\"foo,bar\"");
        assert_eq!(csv_escape("foo\"bar"), "\"foo\"\"bar\"");
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn test_ndjson_and_summary() {
        let report = VerifyReport {
            version: "2.1.0".to_string(),
            command: "verify".to_string(),
            hashfile: "sums.txt".to_string(),
            algorithm: "sha256".to_string(),
            root: ".".to_string(),
            summary: VerifySummary { total: 1, matched: 1, mismatched: 0, errors: 0, missing: 0 },
            exit_code: 0,
            entries: vec![EntryReport {
                path: "a.txt".to_string(),
                status: EntryStatus::Match,
                expected: Some("111".to_string()),
                actual: Some("111".to_string()),
                error: None,
            }],
        };

        let mut buf = Vec::new();
        write_verify_report(&mut buf, OutputFormat::Ndjson, &report, HashMode::Text).unwrap();
        let lines: Vec<&str> = std::str::from_utf8(&buf).unwrap().lines().collect();
        assert_eq!(lines.len(), 2);

        let v1: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(v1["path"], "a.txt");

        let v2: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert!(v2.get("summary").is_some());
        assert_eq!(v2["summary"]["total"], 1);
    }

    #[test]
    fn test_verify_summary_from_entries() {
        let entries = vec![
            EntryReport::new("1.txt", EntryStatus::Match),
            EntryReport::new("2.txt", EntryStatus::Mismatch),
            EntryReport::new("3.txt", EntryStatus::Error),
            EntryReport::new("4.txt", EntryStatus::Missing),
        ];
        let summary = VerifySummary::from_entries(&entries);
        assert_eq!(summary.total, 4);
        assert_eq!(summary.matched, 1);
        assert_eq!(summary.mismatched, 1);
        assert_eq!(summary.errors, 1);
        assert_eq!(summary.missing, 1);
    }
}
