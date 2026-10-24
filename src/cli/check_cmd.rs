// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Check 子命令參數。

use std::path::PathBuf;

use clap::Args;

use crate::algorithm::HashAlgo;

/// 驗證檔案或文字內容的 hash。
#[derive(Args, Debug)]
#[command(group = clap::ArgGroup::new("input").required(true).args(["path", "text"]))]
pub struct CheckArgs {
    /// 要驗證的目標路徑 (與 --text 互斥)
    #[arg(conflicts_with = "text")]
    pub path: Option<PathBuf>,

    /// 要驗證的文字內容 (與 PATH 互斥)
    #[arg(long, conflicts_with = "path")]
    pub text: Option<String>,

    /// 期望的 hash 值 (可重複，任一匹配即通過)
    #[arg(short, long, required = true)]
    pub expect: Vec<String>,

    /// 使用的演算法 (預設 sha256)
    #[arg(short, long)]
    pub algo: Option<HashAlgo>,

    /// 期望的輸出長度 (bytes)
    #[arg(long)]
    pub length: Option<u64>,

    /// hash 比較時忽略大小寫
    #[arg(long)]
    pub ignore_case: bool,
}
