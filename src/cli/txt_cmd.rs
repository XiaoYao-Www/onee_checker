// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! `txt` 子命令 — 生成文字樹狀目錄結構。

use std::path::PathBuf;

use clap::{Args, ValueEnum};

/// 生成文字樹狀目錄結構
///
/// 以樹狀結構展示目錄，可選擇顯示檔案大小、修改時間、建立時間。
#[derive(Args, Clone, Debug)]
pub struct TxtArgs {
    /// 要掃描的目錄路徑
    #[arg(value_hint = clap::ValueHint::DirPath)]
    pub path: PathBuf,

    /// 輸出檔案路徑。省略時自動生成 `<dirname>.tree.txt`。
    /// 指定 `-` 輸出到 stdout。
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// 檔案大小顯示格式
    #[arg(short, long, value_enum)]
    pub size: Option<SizeType>,

    /// 顯示最後修改時間
    #[arg(short = 'm', long)]
    pub modified: bool,

    /// 顯示建立時間
    #[arg(short = 'c', long)]
    pub created: bool,

    /// 安靜模式
    #[arg(short, long)]
    pub quiet: bool,
}

/// 檔案大小顯示類型
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum SizeType {
    /// 1024 為底 (KiB, MiB, ...)
    Binary,
    /// 1000 為底 (KB, MB, ...)
    Decimal,
    /// 原始位元組數
    Raw,
}
