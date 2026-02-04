// 任務類型定義
use std::path::PathBuf;
use std::sync::mpsc;
use std::io;

use super::Hash::{HashData, HashType};



// ####################
// 任務結果
// ####################

/// ### 哈希任務結果
/// 
/// - root_path 根目錄
/// - hash_type 哈希類型
/// - data 哈希資料
pub struct HashResult {
    pub root_path: PathBuf,
    pub hash_type: HashType,
    pub data: Vec<HashData>,
}

/// ### 任務結果類型
///
/// 表示背景任務執行後的統一回傳結果。
///
/// 不同類型的任務會被包裝成同一個列舉型別，方便在任務系統或執行緒之間傳遞與分派處理。
pub enum TaskResult {
    /// ### Hash 驗證完結果
    ///
    /// 成功時 (`Ok`)：
    /// - `Vec<HashData>` 所有檔案的雜湊與中繼資料  
    /// - `PathBuf` 本次驗證的根目錄路徑
    ///
    /// 失敗時 (`Err`)：
    /// - `io::Error` 讀取檔案或存取檔案系統時發生錯誤
    Hash(io::Result<HashResult>),
}


// ####################
// 任務進度
// ####################

/// ### 任務進度類型
///
/// 不同任務有不同進度類型。
/// 
/// 不同進度類型參數可能不同。
pub enum TaskProgress {
    /// ### Hash 任務進度
    /// 
    /// - `usize` 進度總數量
    /// - `Option<usize>` 當前進度完成數量
    Hash {
        total: usize,
        current: Option<usize>,
    },
}


// ####################
// 任務結構
// ####################

/// ### 背景任務結構
///
/// 單個背景任務的訊息介面。
///
/// - progress_rx 用於接收進度更新
/// - result_rx 用於接收最終結果
pub struct Task {
    progress_rx: mpsc::Receiver<TaskProgress>,
    result_rx: mpsc::Receiver<TaskResult>,
}

impl Task {
    /// ### 嘗試接收進度更新
    pub fn tryRecvProgress(&self) -> Result<Option<TaskProgress>, mpsc::TryRecvError> {
        match self.progress_rx.try_recv() {
            Ok(p) => Ok(Some(p)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// ### 嘗試接收最終結果
    pub fn tryRecvResult(&self) -> Result<Option<TaskResult>, mpsc::TryRecvError> {
        match self.result_rx.try_recv() {
            Ok(r) => Ok(Some(r)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e),
        }
    }
}