// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! 結構化錯誤類型 — 所有錯誤都攜帶 exit code，供 CLI 層與 embedder 使用。

use std::path::PathBuf;

use thiserror::Error;

/// ### 工具全域錯誤類型
///
/// 每個變體綁定一個 POSIX 風格 exit code：
/// - `0` — 成功
/// - `1` — hash 不匹配（驗證失敗）
/// - `2` — I/O 或內部錯誤
/// - `3` — 使用者輸入錯誤（CLI 參數、路徑無效等）
#[derive(Error, Debug)]
pub enum OneeError {
    // ── I/O ──────────────────────────────────────────────
    /// 包裝 std::io::Error
    #[error("I/O 錯誤: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化／反序列化錯誤
    #[error("JSON 錯誤: {0}")]
    Serde(#[from] serde_json::Error),

    // ── Hash ─────────────────────────────────────────────
    /// 驗證時 hash 不匹配
    #[error("Hash 不匹配: {path}\n  預期: {expected}\n  實際: {actual}")]
    HashMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },

    /// Hash 檔解析錯誤（格式不符）
    #[error("Hash 檔格式錯誤 (行 {line}): {detail}")]
    HashFileParseError {
        line: usize,
        detail: String,
    },

    // ── 輸入驗證 ─────────────────────────────────────────
    /// 路徑不存在或不是有效檔案/目錄
    #[error("無效路徑: {0}")]
    InvalidPath(String),

    /// 算法不支援自訂長度
    #[error("算法不支援自訂輸出長度: {algorithm}")]
    UnsupportedLength {
        algorithm: String,
    },

    /// 不支援或未知的 hash 算法
    #[error("不支援的算法: {0}")]
    UnsupportedAlgorithm(String),

    /// 其他使用者輸入錯誤
    #[error("參數錯誤: {0}")]
    ArgumentError(String),
}

impl OneeError {
    /// 根據錯誤類型回傳 POSIX exit code
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::HashMismatch { .. } => 1,
            Self::Io(_) | Self::Serde(_) => 2,
            Self::InvalidPath(_)
            | Self::UnsupportedLength { .. }
            | Self::UnsupportedAlgorithm(_)
            | Self::HashFileParseError { .. }
            | Self::ArgumentError(_) => 3,
        }
    }
}

/// 從 `&str` 或 `String` 快速建構 `InvalidPath`
impl From<&str> for OneeError {
    fn from(s: &str) -> Self {
        Self::InvalidPath(s.to_string())
    }
}

/// 從 `String` 建構
impl From<String> for OneeError {
    fn from(s: String) -> Self {
        Self::InvalidPath(s)
    }
}

/// Result 別名，減少重複
pub type Result<T> = std::result::Result<T, OneeError>;
