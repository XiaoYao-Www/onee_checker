// 定義動態哈希介面


/// ### 動態哈希器介面
/// 
/// 定義動態哈希器所需實作的方法。
/// 
/// - update 更新哈希器狀態
/// - finalize 完成哈希計算並返回雜湊值
pub trait DynHasher: Send {
    fn update(&mut self, data: &[u8]);
    fn finalize(self: Box<Self>) -> Vec<u8>;
}