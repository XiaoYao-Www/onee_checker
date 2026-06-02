// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! `json` 子命令 — 生成 JSON 目錄結構紀錄。

use std::path::PathBuf;

use clap::Args;

/// 生成 JSON 目錄結構紀錄
///
/// 遞迴掃描目錄，輸出包含檔案大小、時間戳、符號連結等資訊的 JSON 結構檔。
#[derive(Args, Clone, Debug)]
pub struct JsonArgs {
    /// 要掃描的目錄路徑
    #[arg(value_hint = clap::ValueHint::DirPath)]
    pub path: PathBuf,

    /// 輸出檔案路徑。省略時自動生成 `<dirname>.struct.json`。
    /// 指定 `-` 輸出到 stdout。
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// 安靜模式
    #[arg(short, long)]
    pub quiet: bool,
}
