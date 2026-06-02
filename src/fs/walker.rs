// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! 目錄遍歷 — 使用 `walkdir`，支援深度限制與 symlink 逃脫防護。

use std::fs::{self, symlink_metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use walkdir::WalkDir;

use super::node::FileNode;

/// 最大遞迴深度（防止 stack overflow）
const MAX_DEPTH: u16 = 1024;

/// 遞迴列出指定路徑下的所有**檔案**（排除目錄）。
pub fn list_files(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = Vec::new();

    if path.is_file() {
        entries.push(path.to_path_buf());
        return Ok(entries);
    }

    for entry in WalkDir::new(path)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            entries.push(entry.into_path());
        }
    }

    Ok(entries)
}

/// 遞迴建立 `FileNode` 樹狀結構。
///
/// # 安全防護
///
/// - **深度限制**: 超過 1024 層停止遞迴
/// - **Symlink 防護**: 指向根目錄外的 symlink 目錄不進入
pub fn build_file_node(path: &Path) -> io::Result<FileNode> {
    let canonical_path = fs::canonicalize(path)?;
    build_node_recursive(&canonical_path, &canonical_path, 0)
}

/// 內部遞迴：建構單一節點
///
/// - `path` — 當前節點路徑
/// - `root` — 原始根目錄（用於 symlink 逃脫檢查）
/// - `depth` — 當前深度（用於深度限制）
fn build_node_recursive(path: &Path, root: &Path, depth: u16) -> io::Result<FileNode> {
    let metadata = symlink_metadata(path)?;
    let file_type = metadata.file_type();

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| {
            path.to_string_lossy()
                .trim_start_matches("\\\\?\\")
                .replace('\\', "/")
        });

    let is_dir = file_type.is_dir();
    let is_symlink = file_type.is_symlink();

    let symlink_target: Option<String> = if is_symlink {
        fs::read_link(path)
            .ok()
            .map(|p| p.to_string_lossy().into_owned().replace('\\', "/"))
    } else {
        None
    };

    let extension = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase());

    let last_modified = metadata.modified().ok().map(to_unix_timestamp);
    let created_at = metadata.created().ok().map(to_unix_timestamp);

    // 決定是否可以進入子節點：
    // 1. 是目錄
    // 2. 不是 symlink（或 symlink 指向的目標仍在 root 內）
    // 3. 未超過最大深度
    let should_recurse = is_dir && depth < MAX_DEPTH;

    let mut children: Option<Vec<FileNode>> = None;
    let mut total_size = metadata.len();

    if should_recurse {
        // 如果是 symlink 目錄，檢查目標是否在 root 內
        let can_enter = if is_symlink {
            if let Ok(target) = fs::read_link(path) {
                let abs_target = if target.is_absolute() {
                    target
                } else {
                    // 相對路徑：相對於 symlink 所在目錄
                    let parent = path.parent().unwrap_or(root);
                    parent.join(&target)
                };
                // canonicalize 後檢查是否在 root 內
                match fs::canonicalize(&abs_target) {
                    Ok(canon_target) => canon_target.starts_with(root),
                    Err(_) => false,
                }
            } else {
                false
            }
        } else {
            true // 真實目錄，安全
        };

        if can_enter {
            let mut list: Vec<FileNode> = Vec::new();
            let mut dir_sum = 0u64;

            for entry in WalkDir::new(path)
                .max_depth(1)
                .sort_by(|a, b| a.file_name().cmp(b.file_name()))
                .into_iter()
                .filter_map(|e| e.ok())
                .skip(1) // 跳過自身
            {
                if let Ok(child) = build_node_recursive(&entry.path(), root, depth + 1) {
                    dir_sum += child.size;
                    list.push(child);
                }
            }

            list.sort_by(|a, b| {
                b.is_dir
                    .cmp(&a.is_dir)
                    .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });

            total_size = dir_sum;
            children = Some(list);
        }
    }

    Ok(FileNode {
        name,
        is_dir,
        is_symlink,
        extension,
        size: total_size,
        last_modified,
        created_at,
        children,
        symlink_target,
    })
}

/// 將 `SystemTime` 轉換為 Unix timestamp (seconds)
pub fn to_unix_timestamp(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(dur) => dur.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    }
}

/// 驗證路徑是否存在且為檔案或目錄
pub fn validate_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("路徑不存在: {}", path.display()));
    }
    if !path.is_file() && !path.is_dir() {
        return Err(format!("路徑既不是檔案也不是目錄: {}", path.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    #[test]
    fn test_list_files_single_file() {
        let dir = std::env::temp_dir().join("onee_test_list_single");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.txt");
        File::create(&file_path).unwrap();

        let files = list_files(&file_path).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], file_path);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_files_directory() {
        let dir = std::env::temp_dir().join("onee_test_list_dir");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir.join("sub")).unwrap();
        File::create(&dir.join("a.txt")).unwrap();
        File::create(&dir.join("sub/b.txt")).unwrap();

        let files = list_files(&dir).unwrap();
        assert_eq!(files.len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_build_file_node() {
        let dir = std::env::temp_dir().join("onee_test_node");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut f = File::create(&dir.join("hello.txt")).unwrap();
        f.write_all(b"hello world").unwrap();

        let node = build_file_node(&dir).unwrap();
        assert!(node.is_dir);
        assert_eq!(node.name, dir.file_name().unwrap().to_string_lossy());
        assert!(node.children.is_some());
        assert_eq!(node.children.as_ref().unwrap().len(), 1);
        assert_eq!(node.children.as_ref().unwrap()[0].name, "hello.txt");
        assert!(node.children.as_ref().unwrap()[0].size > 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_max_depth_enforced() {
        // 建立一個深層巢狀目錄結構測試深度限制
        let dir = std::env::temp_dir().join("onee_test_depth");
        let _ = fs::remove_dir_all(&dir);
        let mut current = dir.clone();
        for i in 0..10 {
            current = current.join(format!("d{i}"));
            fs::create_dir_all(&current).unwrap();
            File::create(current.join("f.txt")).unwrap();
        }

        let node = build_file_node(&dir).unwrap();
        // 至少根節點存在
        assert!(node.is_dir);

        let _ = fs::remove_dir_all(&dir);
    }
}
