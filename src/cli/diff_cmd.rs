// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Diff 子命令參數。

use clap::Args;

/// 比較兩份 manifest 檔案的差異。
#[derive(Args, Debug)]
pub struct DiffArgs {
    /// 舊的 manifest 檔案路徑
    pub old: String,

    /// 新的 manifest 檔案路徑
    pub new: String,
}
