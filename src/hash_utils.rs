use std::fs::{File, DirEntry};
use std::path::{PathBuf, Path};
use std::io::{self, Read, Write};
use std::sync::mpsc::Sender;
use sha2::{Sha256, Digest};

pub struct ShaData {
    pub path: PathBuf,
    pub sha: String,
    pub sha_type: String,
}

/// 進度回報的訊息類型
pub enum ProgressMessage {
    Starting(usize), // 開始，回報總檔案數
    Hashing(usize, String), // 正在處理第 n 個檔案，檔名
    Finished, // 全部完成
}

/// ### 計算選取檔案或資料夾的雜湊值
/// 
/// 目前用sha256，回傳(是否為資料夾, 雜湊結果)
/// 
/// - path 檔案或資料夾路徑
/// - progress_sender 用於回報進度的發送端
pub fn hash_selected(path: &Path, progress_sender: Sender<ProgressMessage>) -> io::Result<(bool, Vec<ShaData>)> {
    let mut result: Vec<ShaData> = Vec::new();

    // 🔑 以輸入路徑的父目錄作為相對根
    let base_dir: &Path = path.parent().unwrap_or(path);

    if path.is_dir() {
        let mut files: Vec<PathBuf> = Vec::new();
        collect_files(path, &mut files)?;
        files.sort();

        let total_files = files.len();
        if progress_sender.send(ProgressMessage::Starting(total_files)).is_err() {
            // 如果發送失敗 (例如主執行緒已關閉接收端)，就直接返回
            return Ok((true, result));
        }

        for (i, file_path) in files.iter().enumerate() {
            if file_path.is_file() {
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if progress_sender.send(ProgressMessage::Hashing(i + 1, file_name)).is_err() {
                    break; // 中斷迴圈
                }
                let hash: String = compute_file_hash(&file_path)?;

                // 相對於輸入路徑的父目錄
                let relative_path: PathBuf = file_path
                    .strip_prefix(base_dir)
                    .map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("path {:?} is not under base dir {:?}", file_path, base_dir),
                        )
                    })?
                    .to_path_buf();

                result.push(ShaData {
                    path: relative_path,
                    sha: hash,
                    sha_type: "SHA256".to_string(),
                });
            }
        }

        let _ = progress_sender.send(ProgressMessage::Finished);
        Ok((true, result))
    } else if path.is_file() {
        if progress_sender.send(ProgressMessage::Starting(1)).is_err() {
            return Ok((false, result));
        }

        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if progress_sender.send(ProgressMessage::Hashing(1, file_name)).is_err() {
            return Ok((false, result));
        }

        let hash: String = compute_file_hash(path)?;

        let relative_path = path
            .strip_prefix(base_dir)
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("path {:?} is not under base dir {:?}", path, base_dir),
                )
            })?
            .to_path_buf();

        result.push(ShaData {
            path: relative_path,
            sha: hash,
            sha_type: "SHA256".to_string(),
        });

        let _ = progress_sender.send(ProgressMessage::Finished);
        Ok((false, result))
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "path is neither file nor directory",
        ))
    }
}

/// ### 遞迴所有檔案路徑
/// 
/// 遞迴收集目錄底下的所有檔案路徑
/// 
/// - dir 資料夾路徑
/// - files 儲存路徑的向量
fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry: DirEntry = entry?;
        let path:PathBuf = entry.path();
        // 使用 file_type() 避免跟隨符號連結造成無限遞迴 (Stack Overflow)
        if entry.file_type()?.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

/// ### 計算單一檔案的雜湊值
/// 
/// 使用 sha256 雜湊算法計算檔案的雜湊值
/// 
/// - path 檔案路徑
fn compute_file_hash(path: &Path) -> io::Result<String> {
    let mut file: File = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer: [u8; 4096] = [0u8; 4096];
    // 讀取檔案內容並更新雜湊器
    loop {
        let n: usize = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}


fn format_checksum_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// ### 儲存雜湊結果至檔案
/// 
/// 將雜湊結果寫入指定檔案，格式相容於 sha256sum -c 等工具
/// 格式為: <hash> *<relative_path>
/// 
/// - data 雜湊資料列表
/// - output_file 輸出檔案路徑
pub fn save_checksums(data: &[ShaData], output_file: &Path) -> io::Result<()> {
    let mut file: File = File::create(output_file)?;
    for entry in data {
        // 使用 * 表示二進位模式，這是多數雜湊驗證工具的預設或相容格式
        writeln!(file, "{} *{}", entry.sha, format_checksum_path(&entry.path))?;
    }
    Ok(())
}
