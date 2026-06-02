// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! BLAKE3 哈希器（可變輸出長度）
//!
//! # 多線程
//!
//! 啟用 `rayon` feature 後，`blake3::hash()` 內部使用多線程樹狀 hash。
//! 對於大於 256 MiB 的檔案，這比串流 `update()` 快 3-8x。
//!
//! 由於 `update_with_join()` 是 blake3 crate 的私有方法，
//! 多線程僅能透過 `blake3::hash()`（固定 32 bytes）或
//! 串流 `update()` 使用。我們對預設長度啟用 `hash()` 路徑。

/// BLAKE3 哈希器（可變輸出長度支援）
#[derive(Clone)]
pub struct Blake3Hasher {
    inner: blake3::Hasher,
    out_len: u16,
}

impl Blake3Hasher {
    #[must_use]
    pub fn new(out_len: u16) -> Self {
        Self { inner: blake3::Hasher::new(), out_len }
    }

    /// 串流式更新（單線程）
    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    /// 完成 hash 並回傳指定長度的輸出
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        let mut buf = vec![0u8; self.out_len as usize];
        self.inner.finalize_xof().fill(&mut buf);
        buf
    }
}

/// 對整個 buffer 計算 BLAKE3 hash，預設長度（32 bytes）時啟用多線程。
///
/// 使用 `blake3::hash()`（內部透過 `rayon` 多線程樹狀 hash）。
/// 非預設長度時使用串流模式。
#[must_use]
pub fn blake3_hash_bulk(data: &[u8], out_len: u16) -> Vec<u8> {
    if out_len == 32 {
        // 多線程路徑：blake3::hash() 內部使用 std threads 或 rayon
        blake3::hash(data).as_bytes().to_vec()
    } else {
        // 自訂長度：串流路徑
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let mut buf = vec![0u8; out_len as usize];
        hasher.finalize_xof().fill(&mut buf);
        buf
    }
}
