// 檔案系統類型定義
use serde::Serialize;


/// ### 檔案節點
///
/// - name 節點名稱
/// - is_dir 是否是資料夾
/// - size 節點大小(資料夾是其內所有檔案的綜合大小)
/// - last_modified 最後編輯日期(文字顯示)
/// - children 子節點
#[derive(Serialize, Clone)]
pub struct FileNode {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub last_modified: u64,
    pub children: Option<Vec<FileNode>>,
}