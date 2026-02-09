// 檔案系統類型定義
use serde::{self, Deserialize, Serialize};

/// ### 檔案節點
///
/// - name 節點名稱
/// - is_dir 是否是資料夾
/// - is_symlink 是否是符號連結
/// - extension 擴展名稱
/// - size 節點大小(資料夾是其內所有檔案的綜合大小)
/// - last_modified 最後編輯日期( UNIX timestamp )
/// - created_at 創建日期( UNIX timestamp )
/// - children 子節點
/// - symlink_target 符號連結的目標
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileNode {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub extension: Option<String>,
    pub size: u64,
    pub last_modified: Option<u64>,
    pub created_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink_target: Option<String>,
}
