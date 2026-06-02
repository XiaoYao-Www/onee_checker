// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! 檔案系統節點類型 — `FileNode` 與 `FileNodeContainer`。

use serde::{Deserialize, Serialize};

/// ### 檔案節點
///
/// 遞迴目錄結構中的單一節點，可為檔案或目錄。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileNode {
    /// 節點名稱
    pub name: String,
    /// 是否為目錄
    pub is_dir: bool,
    /// 是否為符號連結
    pub is_symlink: bool,
    /// 副檔名（檔案才有）
    pub extension: Option<String>,
    /// 大小（bytes）。目錄為其所有子節點的總和
    pub size: u64,
    /// 最後修改時間 (Unix timestamp, seconds)
    pub last_modified: Option<i64>,
    /// 建立時間 (Unix timestamp, seconds)
    pub created_at: Option<i64>,
    /// 子節點（目錄才有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
    /// 符號連結目標
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink_target: Option<String>,
}

/// 檔案節點容器 — JSON 輸出結構
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileNodeContainer {
    /// 版本號
    pub version: String,
    /// 生成時間 (Unix timestamp, seconds)
    pub generation_time: i64,
    /// 節點根列表
    pub nodes: Vec<FileNode>,
}
