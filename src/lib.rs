// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! # Onee Checker — 專業 hash 驗證與目錄結構工具
//!
//! 既可作為 CLI 工具 (`oneechk`)，也可作為 Rust library 由其他專案嵌入使用。
//!
//! ## Library 使用範例
//!
//! ```rust,ignore
//! use onee_checker::prelude::*;
//! use std::path::Path;
//!
//! // 計算單一檔案的 SHA-256
//! let mut buf = vec![0u8; 64 * 1024];
//! let hash = compute_file_hash(Path::new("/path/to/file"), &HashType::SHA256, &mut buf)?;
//!
//! // 並行掃描整個目錄
//! let files = list_files(Path::new("/some/dir"))?;
//! let results = compute_hashes_parallel(&files, &HashType::SHA256, 64 * 1024);
//! ```

pub mod algorithm;
pub mod cli;
pub mod error;
pub mod fs;
pub mod hash;
pub mod hasher;
pub mod tree;

/// 為 embedder 提供的便利 prelude
pub mod prelude {
    pub use crate::algorithm::{BufferSize, HashAlgo, HashData, HashType};
    pub use crate::error::{OneeError, Result};
    pub use crate::fs::{
        build_file_node, canonicalize_root, list_files, parse_hash_file, sanitize_rel_path,
        save_hash_file, validate_path, FileNode, HashEntry,
    };
    pub use crate::hash::{
        compute_file_hash, compute_hash_reader, compute_hashes_parallel,
        compute_multi_hashes_parallel, verify_file_hash, verify_hash_file,
    };
    pub use crate::hasher::{blake3_hash_bulk, HasherEnum};
    pub use crate::tree::{write_tree, SizeFormat, TreeOption};
}
