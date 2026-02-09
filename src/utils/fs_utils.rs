// 檔案系統函式庫
use std::ffi::OsStr;
use std::fs::{self, DirEntry, File, FileType, Metadata, symlink_metadata};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf, StripPrefixError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::types::FS::FileNode;
use crate::types::Hash::HashData;

/// ### 列出所有檔案
///
/// 列出輸入路徑所代表的所有檔案。
///
/// - path 目標路徑
pub fn list_file(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = Vec::new();

    // 1. 基本路徑檢查：如果是檔案，直接回傳
    if path.is_file() {
        entries.push(path.to_path_buf());
        return Ok(entries);
    }

    // 2. 遞迴蒐集檔案
    fn collect_recursive(dir: &Path, all_files: &mut Vec<PathBuf>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry: DirEntry = entry?;
                let p: PathBuf = entry.path();
                if p.is_dir() {
                    collect_recursive(&p, all_files)?; // 遞迴進去
                } else {
                    all_files.push(p); // 檔案才加入
                }
            }
        }
        Ok(())
    }

    collect_recursive(path, &mut entries)?;

    // 3. 統一排序 (只需排一次，效率最高)
    // 建議根據完整路徑排序，避免不同目錄下的同名檔案亂掉
    entries.sort_by(|a: &PathBuf, b: &PathBuf| {
        a.to_string_lossy()
            .to_lowercase()
            .cmp(&b.to_string_lossy().to_lowercase())
    });

    Ok(entries)
}

/// 輔助函數：將 SystemTime 轉為 Unix Timestamp
fn to_unix_timestamp(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(dur) => {
           dur.as_secs() as i64
        }
        Err(e) => {
            let dur: Duration = e.duration();
            -(dur.as_secs() as i64)
        }
    }
}

/// ### 創建檔案節點:遞迴調用
///
/// 根據指定路徑創建節點結構。
///
/// - path 路徑
fn build_file_node_recursive(path: &Path) -> io::Result<FileNode> {
    // 使用 symlink_metadata 而非 metadata，這樣才能抓到連結本身而非目標
    let metadata: Metadata = symlink_metadata(path)?;
    let file_type: FileType = metadata.file_type();

    let name: String = path
        .file_name()
        .map(|n: &OsStr| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| {
            path.to_string_lossy()
                .replace("\\\\?\\", "") // 移除 Windows UNC 前綴
                .replace("\\", "/") // 統一路徑斜線
        });

    let is_dir: bool = file_type.is_dir();
    let is_symlink: bool = file_type.is_symlink();

    // 抓取符號連結指向
    let symlink_target: Option<String> = if is_symlink {
        fs::read_link(path)
            .ok()
            .map(|p: PathBuf| p.to_string_lossy().into_owned().replace("\\", "/"))
    } else {
        None
    };

    // 取得副檔名
    let extension: Option<String> = path
        .extension()
        .map(|ext: &OsStr| ext.to_string_lossy().to_lowercase());

    // 取得時間 (封裝成輔助函數)
    let last_modified: Option<i64> = metadata.modified().ok().map(to_unix_timestamp);
    let created_at: Option<i64> = metadata.created().ok().map(to_unix_timestamp);

    let mut children: Option<Vec<FileNode>> = None;
    let mut total_size: u64 = metadata.len();

    // 只有是「真正」的資料夾才遞迴（如果是指向資料夾的連結，則不進入，防止死循環）
    if is_dir && !is_symlink {
        let mut list: Vec<FileNode> = Vec::new();
        let mut dir_sum_size: u64 = 0;

        // 讀取資料夾內容
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                // 遞迴調用
                if let Ok(child_node) = build_file_node_recursive(&entry.path()) {
                    dir_sum_size += child_node.size;
                    list.push(child_node);
                }
            }
        }

        // 排序：資料夾在前，檔案在後，並按名稱排序
        list.sort_by(|a: &FileNode, b: &FileNode| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        total_size = dir_sum_size;
        children = Some(list);
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

/// ### 創建檔案節點
///
/// 會處理 Windows 路徑長度限制
///
/// - path 路徑
pub fn build_file_node(path: &Path) -> io::Result<FileNode> {
    // 1. 轉換為絕對路徑且處理 Windows 長路徑限制 (UNC prefix)
    let canonical_path: PathBuf = fs::canonicalize(path)?;

    // 2. 開始遞迴遍歷
    build_file_node_recursive(&canonical_path)
}

/// ### 儲存雜湊結果至檔案
///
/// 將雜湊結果寫入指定檔案
/// 格式為: <hash> *<relative_path>
///
/// - data 雜湊資料列表
/// - outputFile 輸出檔案路徑
pub fn save_hash_to_file(
    data: &[HashData],
    output_file: &Path,
    root_path: &Path,
) -> io::Result<()> {
    let mut file: File = File::create(output_file)?;
    for entry in data {
        let rel_path: &Path = entry
            .path
            .strip_prefix(root_path)
            .map_err(|e: StripPrefixError| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        // 轉換路徑分隔符為 /
        let path_str: String = rel_path.to_string_lossy().replace('\\', "/");
        // 寫入驗證資訊，使用 * 表示二進位模式
        writeln!(file, "{} *{}", entry.hash_hex(), path_str)?;
    }
    Ok(())
}

/// ### 驗證路徑有效性
///
/// 驗證路徑是否為存在的檔案或資料夾
///
/// - path 驗證的路徑
pub fn validate_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("路徑不存在: {}", path.display()));
    }

    if !path.is_file() && !path.is_dir() {
        return Err(format!("路徑不是文件也不是目錄: {}", path.display()));
    }

    Ok(())
}

/// ### 儲存FileNode為檔案
///
/// 將FileNode儲存為檔案，會有縮排
///
/// - node 節點根
/// - output_path 儲存路徑
pub fn save_file_node_to_file(node: &FileNode, output_path: &Path) -> io::Result<()> {
    let file: File = File::create(output_path)?;

    // 使用 BufWriter 提高大量寫入時的效能
    let writer: BufWriter<File> = BufWriter::new(file);

    // 序列化並寫入
    serde_json::to_writer_pretty(writer, node)
        .map_err(|e: serde_json::Error| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}
