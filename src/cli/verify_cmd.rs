// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! `verify` 子命令 — 驗證既有 hash 驗證檔。

use std::path::PathBuf;

use clap::Args;

use crate::algorithm::{BufferSize, HashAlgo};

/// 驗證 hash 驗證檔
///
/// 解析 hash 驗證檔，逐一比對檔案 hash 是否匹配。
/// 演算法預設從副檔名推斷；亦可手動指定。
#[derive(Args, Clone, Debug)]
pub struct VerifyArgs {
    /// hash 驗證檔路徑（如 `dir.sha256`）
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub hashfile: PathBuf,

    /// 手動指定演算法（預設從副檔名推斷）
    #[arg(short, long, value_enum)]
    pub algo: Option<HashAlgo>,

    /// 根目錄路徑（預設為 hash 檔所在目錄）
    #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
    pub root: Option<PathBuf>,

    /// 讀取緩衝區大小，支援 K/M/G 後綴。預設 1M
    #[arg(long, default_value = "1M", value_parser = clap::value_parser!(BufferSize))]
    pub buffer: BufferSize,

    /// 並行線程數量。預設為 CPU 邏輯核心數
    #[arg(short, long)]
    pub threads: Option<usize>,

    /// 安靜模式 — 僅顯示最終結果
    #[arg(short, long)]
    pub quiet: bool,
}
