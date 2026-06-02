// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! `hash` 子命令 — 計算檔案 hash 並輸出驗證檔。

use std::path::PathBuf;

use clap::Args;

use crate::algorithm::{BufferSize, HashAlgo};

/// 生成 hash 驗證檔
///
/// 計算指定路徑下所有檔案的 hash 值，輸出相容於 shasum 格式的驗證檔。
/// 支援 MD5、SHA-1/2/3、SHAKE、BLAKE2/3 等 14 種演算法。
#[derive(Args, Clone, Debug)]
pub struct HashArgs {
    /// 要計算的路徑（檔案或目錄）
    #[arg(value_hint = clap::ValueHint::AnyPath)]
    pub path: PathBuf,

    /// 演算法（可多次指定，例如 `-a sha256 -a blake3`）
    #[arg(short, long, value_enum)]
    pub algo: Vec<HashAlgo>,

    /// 輸出檔案路徑。省略時自動生成 `<dirname>.<algo_suffix>`。
    /// 指定 `-` 輸出到 stdout。
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// 可變長度演算法的輸出長度（bytes）
    ///
    /// 僅 SHAKE128/256 與 BLAKE3 支援此參數。
    #[arg(long)]
    pub length: Option<u16>,

    /// 讀取緩衝區大小，支援 K/M/G 後綴（如 `4K`、`1M`）。預設 1M
    #[arg(long, default_value = "1M", value_parser = clap::value_parser!(BufferSize))]
    pub buffer: BufferSize,

    /// 並行線程數量。預設為 CPU 邏輯核心數
    #[arg(short, long)]
    pub threads: Option<usize>,

    /// 安靜模式 — 不顯示進度條
    #[arg(short, long)]
    pub quiet: bool,
}
