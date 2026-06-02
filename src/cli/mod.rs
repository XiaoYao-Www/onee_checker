// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! CLI 參數定義 — 使用 clap derive。

mod hash_cmd;
mod verify_cmd;
mod json_cmd;
mod txt_cmd;

pub use hash_cmd::*;
pub use verify_cmd::*;
pub use json_cmd::*;
pub use txt_cmd::*;

use clap::{Parser, Subcommand};

/// Onee Checker — 專業 hash 驗證與目錄結構工具
#[derive(Parser)]
#[command(
    name = "oneechk",
    author,
    version,
    about = "Hash verification & directory structure tool",
    arg_required_else_help = true,
    subcommand_required = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 生成 hash 驗證檔
    Hash(HashArgs),

    /// 驗證 hash 驗證檔
    Verify(VerifyArgs),

    /// 生成 JSON 目錄結構紀錄
    Json(JsonArgs),

    /// 生成文字樹狀目錄結構
    Txt(TxtArgs),
}
