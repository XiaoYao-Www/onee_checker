# 架構文件

## 設計哲學

Onee Checker 遵循 **Library-First** 設計：核心邏輯存在於 `lib.rs`，`main.rs` 僅為薄 CLI 調度層。這確保：

- 其他 Rust 專案可以 `use onee_checker::*` 直接調用
- 外部工具可以 subprocess 方式調用 CLI
- 測試可直接對 library 層執行，無需 CLI 層介入

---

## 模組架構

```
                    ┌──────────────┐
                    │   main.rs    │  CLI 調度層 (parse → dispatch → exit code)
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │    cli/      │  Clap derive 參數定義
                    └──────┬───────┘
                           │
┌──────────────────────────▼──────────────────────────┐
│                      lib.rs                          │
│  公開 API ─ prelude ─ 所有模組匯出                    │
├────────────┬──────────┬──────────┬──────────────────┤
│ algorithm  │  hash    │  hasher  │  tree            │
│ (類型定義) │ (計算)    │ (14算法) │  (文字輸出)       │
├────────────┴──────────┴──────────┴──────────────────┤
│                      fs/                             │
│  ├── node.rs    (FileNode, FileNodeContainer)         │
│  ├── walker.rs  (walkdir 遍歷 + 深度限制 + symlink)   │
│  ├── writer.rs  (Hash 檔/JSON IO + HashEntry parser) │
│  └── path_safe.rs (Path Traversal 防護)              │
├──────────────────────────────────────────────────────┤
│                    error.rs                          │
│             OneeError (thiserror) + exit code        │
└──────────────────────────────────────────────────────┘
```

---

## 資料流

### Hash 計算流程

```
CLI: hash <PATH>
    │
    ▼
main.rs: cmd_hash()
    │
    ├─► fs::validate_path()      — 路徑驗證
    ├─► HashAlgo::to_hash_type() — 算法轉換
    ├─► fs::list_files()         — walkdir 收集檔案
    ├─► hash::compute_hashes_parallel()  — rayon 並行
    │       │
    │       ├─► map_init: (buffer, HasherEnum template)
    │       ├─► for each file: template.clone() → update → finalize
    │       └─► collect: Vec<HashData>
    │
    └─► fs::save_hash_file()     — 寫入驗證檔
```

### Verify 流程

```
CLI: verify <HASHFILE>
    │
    ▼
main.rs: cmd_verify()
    │
    ├─► HashAlgo::from_suffix()   — 演算法推斷
    ├─► fs::canonicalize_root()   — 鎖定根目錄
    ├─► fs::parse_hash_file()     — 解析 hash 檔
    │       │
    │       ├─► HashEntry::new()   — hex 驗證
    │       └─► 行數上限檢查 (10M)
    │
    └─► hash::verify_hash_file()  — rayon 並行驗證
            │
            └─► for each entry:
                ├─► fs::sanitize_rel_path() — Path traversal 防護
                ├─► hash::compute_file_hash() — 重新計算
                └─► compare hex strings
```

### JSON/TXT 流程

```
CLI: json|txt <PATH>
    │
    ▼
main.rs: cmd_json()|cmd_txt()
    │
    ├─► fs::build_file_node()     — walkdir 遞迴建構 FileNode
    │       │
    │       ├─► symlink_metadata() — 區分檔案/目錄/symlink
    │       ├─► symlink escape check — 目標需在 root 內
    │       └─► depth check — 上限 1024 層
    │
    ├─► serde_json::to_writer_pretty() (json)
    └─► tree::write_tree()              (txt)
```

---

## Hasher 架構演進

### 舊架構 (已移除)

```
HashType::create_hasher() → Box<dyn DynHasher>
    ├─ 逐檔 heap alloc (1M files = 1M allocs)
    └─ vtable dispatch (無法 inline)
```

### 新架構 (HasherEnum)

```
HashType::create_hasher() → HasherEnum
    ├─ Stack-allocated flat enum (零 heap alloc)
    ├─ clone() 供 map_init 複本 (64-200 bytes copy)
    ├─ update()/finalize() 為 match 靜態分發
    └─ 編譯器可完全 inline 所有方法鏈
```

```
HasherEnum
├── Md5(Md5Hasher)        → md5::Md5
├── Sha1(Sha1Hasher)      → sha1::Sha1
├── Sha224(Sha224Hasher)  → sha2::Sha224
├── Sha256(Sha256Hasher)  → sha2::Sha256
├── Sha384(Sha384Hasher)  → sha2::Sha384
├── Sha512(Sha512Hasher)  → sha2::Sha512
├── Sha3_256(...)         → sha3::Sha3_256
├── Sha3_512(...)         → sha3::Sha3_512
├── Shake128(...)         → sha3::Shake128 + out_len: u16
├── Shake256(...)         → sha3::Shake256 + out_len: u16
├── Blake2s256(...)       → blake2::Blake2s256
├── Blake2b512(...)       → blake2::Blake2b512
└── Blake3(...)           → blake3::Hasher + out_len: u16
```

---

## 並行模型

### Rayon Parallel Iterator

```rust
files.par_iter().map_init(
    || (vec![0u8; buffer_size], hash_type.create_hasher()),
    |(buf, template), path| {
        let mut hasher = template.clone();
        // ... compute ...
    },
).collect()
```

- **map_init**: 每個 worker thread 初始化一次 (buffer + hasher template)
- **template.clone()**: 每個檔案複製一份獨立 hasher
- **無共享可變狀態**: buffer 和 hasher 都是 thread-local
- **collect()**: 收集結果為 `Vec`，排序在單線程進行

### ProgressBar 同步

```rust
for result in results {
    // ... handle result ...
    if let Some(ref bar) = pb { bar.inc(1); }
}
```

- `indicatif::ProgressBar` 內部使用 `AtomicU64`
- `inc(1)` 是原子操作，無需 Mutex
- 遞增發生在 collect 後的單線程 for-loop 中

---

## 錯誤處理策略

### 分層錯誤

| 層級 | 錯誤類型 | 處理方式 |
|---|---|---|
| 個別檔案 | `io::Error` | 包裝進 `OneeError::Io`，不中斷其他檔案 |
| Hash 行解析 | `HashFileParseError` | 跳過該行，寫入 stderr 警告 |
| Path 驗證 | `InvalidPath` | 拒絕該路徑，記為 error |
| 整體流程 | `HashMismatch` | 累積不匹配計數，最終 exit code 1 |
| 致命錯誤 | `Io` / `Serde` | 立即退出，exit code 2 |

### Graceful Degradation

- hash 檔中格式錯誤的行 → 跳過，繼續處理後續行
- 個別檔案無法讀取 → 記錄錯誤，繼續處理其他檔案
- 目錄中無檔案 → 警告但不報錯
