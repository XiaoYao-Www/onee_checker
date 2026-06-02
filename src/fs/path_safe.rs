// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! 路徑安全防護 — Path Traversal 預防、正規化、邊界檢查。
//!
//! # 防護原則
//!
//! 1. **所有外部輸入的路徑**都必須先通過 `sanitize_rel_path`
//! 2. **根目錄**在每次 verify 開始前先 `canonicalize_root` 鎖定
//! 3. **拒絕** `..` 元件、絕對路徑、`\0` null byte

use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::error::{OneeError, Result};

/// 正規化並鎖定根目錄。
///
/// 將輸入路徑解析為絕對路徑，追蹤 symlink 後回傳真實路徑。
/// 後續所有相對路徑都將以此為基準。
pub fn canonicalize_root(root: &Path) -> Result<PathBuf> {
    let canonical = fs::canonicalize(root)
        .map_err(|e| OneeError::InvalidPath(format!("無法正規化根目錄 {}: {e}", root.display())))?;

    if !canonical.is_dir() {
        return Err(OneeError::InvalidPath(format!("根目錄不是目錄: {}", canonical.display())));
    }

    Ok(canonical)
}

/// 消毒並驗證相對路徑，確保不逃脫根目錄。
///
/// # 檢查項目
///
/// 1. **null byte** — 拒絕 `\0`（底層 C 截斷攻擊）
/// 2. **絕對路徑** — 拒絕以 `/` 或 Windows 磁碟代號開頭
/// 3. **`..` 元件** — 拒絕任何 `..` 字樣（含編碼繞過如 `....//`）
/// 4. **邊界驗證** — join 到根目錄後 canonicalize，確認仍在根目錄內
pub fn sanitize_rel_path(rel_path: &str, root: &Path) -> Result<PathBuf> {
    // 1. 拒絕空路徑
    if rel_path.trim().is_empty() {
        return Err(OneeError::InvalidPath("相對路徑為空".into()));
    }

    // 2. 拒絕 null byte
    if rel_path.contains('\0') {
        return Err(OneeError::InvalidPath("路徑包含 null byte".into()));
    }

    // 3. 拒絕絕對路徑
    let trimmed = rel_path.trim();
    if trimmed.starts_with('/') {
        return Err(OneeError::InvalidPath(format!("拒絕絕對路徑: {rel_path}")));
    }

    // Windows: 拒絕磁碟代號根 (C:\) 或 UNC (\\server)
    #[cfg(target_os = "windows")]
    {
        if trimmed.starts_with('\\') {
            return Err(OneeError::InvalidPath(format!("拒絕絕對路徑: {rel_path}")));
        }
        if trimmed.len() >= 2
            && trimmed.as_bytes()[1] == b':'
            && trimmed.as_bytes()[0].is_ascii_alphabetic()
        {
            return Err(OneeError::InvalidPath(format!("拒絕絕對路徑: {rel_path}")));
        }
    }

    // 4. 拒絕 `..` 元件
    for component in Path::new(trimmed).components() {
        if matches!(component, Component::ParentDir) {
            return Err(OneeError::InvalidPath(format!("拒絕路徑逃脫 (..): {rel_path}")));
        }
    }

    // 5. 拒絕 Windows 裝置名稱
    #[cfg(target_os = "windows")]
    {
        let lower = trimmed.to_lowercase();
        if lower.contains("con")
            || lower.contains("nul")
            || lower.contains("prn")
            || lower.contains("aux")
            || lower.contains("com1")
            || lower.contains("com2")
            || lower.contains("lpt1")
            || lower.contains("lpt2")
        {
            return Err(OneeError::InvalidPath(format!("拒絕 Windows 保留裝置名稱: {rel_path}")));
        }
    }

    // 6. 組合路徑
    let joined = root.join(trimmed);

    // 如果路徑存在 → canonicalize 並確認在 root 內
    if joined.exists() {
        let canonical = fs::canonicalize(&joined).map_err(OneeError::Io)?;
        if !canonical.starts_with(root) {
            return Err(OneeError::InvalidPath(format!("路徑逃脫根目錄: {rel_path}")));
        }
    } else {
        // 路徑不存在 → 保守檢查祖先鏈
        for ancestor in joined.ancestors().skip(1) {
            if ancestor.exists() {
                if let Ok(canon) = fs::canonicalize(ancestor) {
                    if !canon.starts_with(root) {
                        return Err(OneeError::InvalidPath(format!("路徑逃脫根目錄: {rel_path}")));
                    }
                }
                break;
            }
        }
    }

    Ok(joined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    #[test]
    fn test_sanitize_normal_path() {
        let dir = std::env::temp_dir().join("onee_safe_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir.join("sub")).unwrap();
        File::create(&dir.join("sub/file.txt")).unwrap();

        let root = canonicalize_root(&dir).unwrap();
        assert!(sanitize_rel_path("sub/file.txt", &root).is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sanitize_rejects_dotdot() {
        let dir = std::env::temp_dir().join("onee_safe_test2");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let root = canonicalize_root(&dir).unwrap();

        assert!(sanitize_rel_path("../../etc/passwd", &root).is_err());
        assert!(sanitize_rel_path("sub/../../etc", &root).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sanitize_rejects_absolute() {
        let dir = std::env::temp_dir().join("onee_safe_test3");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let root = canonicalize_root(&dir).unwrap();

        assert!(sanitize_rel_path("/etc/passwd", &root).is_err());
        assert!(sanitize_rel_path("\0hidden", &root).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sanitize_rejects_empty() {
        let dir = std::env::temp_dir().join("onee_safe_test4");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let root = canonicalize_root(&dir).unwrap();

        assert!(sanitize_rel_path("", &root).is_err());
        assert!(sanitize_rel_path("  ", &root).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    /// 路徑含空格應被接受
    #[test]
    fn test_sanitize_with_spaces() {
        let dir = std::env::temp_dir().join("onee_test_spaces");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("my file.txt"), b"test").unwrap();
        let root = canonicalize_root(&dir).unwrap();

        assert!(sanitize_rel_path("my file.txt", &root).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    /// Unicode 路徑應被接受
    #[test]
    fn test_sanitize_unicode_path() {
        let dir = std::env::temp_dir().join("onee_test_unicode");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("ファイル.txt"), b"test").unwrap();
        let root = canonicalize_root(&dir).unwrap();

        assert!(sanitize_rel_path("ファイル.txt", &root).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_canonicalize_root_nonexistent() {
        let bad_path = Path::new("/nonexistent_path_xyz123");
        assert!(canonicalize_root(bad_path).is_err());
    }
}
