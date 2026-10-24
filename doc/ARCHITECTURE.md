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
                    │  hash/verify │  （2.1.0 起新增 check/diff）
                    └──────┬───────┘
                           │
┌──────────────────────────▼──────────────────────────┐
│                      lib.rs                          │
│  公開 API ─ prelude ─ 所有模組匯出                    │
├────────────┬──────────┬──────────┬──────────────────┤
│ algorithm  │  hash    │  hasher  │  tree            │
│ (類型定義) │ (計算)    │ (14算法) │  (文字輸出)       │
├────────────┼──────────┴──────────┼──────────────────┤
│ report     │  manifest           │  （2.1.0 新增）  │
│ (輸出契約) │  (清單解析／diff)    │                  │
├────────────┴─────────────────────┴──────────────────┤
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

`report` 與 `manifest` 是 2.1.0 的兩個新增切面：

- **report**：只負責「把已知結果寫成某種格式」，不讀檔、不算雜湊。CLI 的 `--format` 與
  `verify --report` 都透過它輸出；embedder 可直接重用，保證與 CLI 的 JSON schema 一致。
- **manifest**：只負責「把文字清單變成 `HashEntry`」與「比對兩份清單」。
  它不知道檔案在哪裡，也不知道雜湊怎麼算，因此可以在單元測試中完全用字串驅動。

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
    ├─► root 決定：-r 或清單所在目錄（空字串時退回 "."）
    ├─► root.canonicalize()       — 統一絕對路徑，讓 entry 可轉相對路徑
    ├─► fs::parse_hash_file_strict() — 嚴格解析（格式錯誤 → exit 3）
    │       │
    │       ├─► HashEntry::new()   — hex 驗證
    │       └─► 行數上限檢查 (10M)
    │
    ├─► hash::verify_hash_file()  — rayon 並行驗證
    │       │
    │       └─► for each entry:
    │           ├─► fs::sanitize_rel_path() — Path traversal 防護
    │           ├─► hash::compute_file_hash() — 重新計算
    │           └─► compare hex strings
    │
    ├─► 組裝 report::VerifyReport（entries + summary）
    ├─► --format 非 human → report::write_verify_report(stdout)
    ├─► --report <PATH>   → 額外寫一份 JSON
    └─► exit code = report.compute_exit_code()（0/1/2/4）
```

2.1.0 的關鍵改變：**先嚴格解析、再驗證**。清單本身有問題時直接回 `3`，
不會出現「檔案全部不符」這種誤導性結果。

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

## 並行模型優化

### 多演算法單次 I/O

**問題**：`cmd_hash` 為每個 `-a` 參數走完整管線（`list_files` → `compute_hashes_parallel` → save），
導致 `-a sha256 -a blake3` 重複讀取同一批檔案。

**解決方案**：`compute_multi_hashes_parallel()` (新增於 v2.1)

```rust
pub fn compute_multi_hashes_parallel(
    files: &[PathBuf],
    hash_types: &[HashType],
    buffer_size: usize,
) -> Vec<Vec<Result<HashData, OneeError>>>
```

- 每個 worker 配置 N 個 hasher（每個演算法一個） + 共用 buffer
- 一次 `File::open` + 一次 read loop，同時 update 所有 hasher
- 回傳轉置後的 `Vec<Vec<...>>`（外層 per-algorithm，內層 per-file）
- 當 CLI 指定多個 `-a` 時自動啟用，單一演算法維持既有路徑

**收益**：N 種演算法時 I/O 從 Nx 降為 1x

### BLAKE3 大檔案多線程

**問題**：`blake3::Hasher::update()` 是純單線程串流，
對大檔案無法利用 BLAKE3 的樹狀 hash 多線程能力。

**解決方案**：`blake3_hash_bulk()` + `compute_blake3_bulk_path()`

- 檢測條件：`hash_type == BLAKE3(_)` 且 256 MiB ≤ 檔案大小 ≤ 1 GiB
- 將整個檔案載入記憶體，使用 `blake3::hash()`（內部多線程）或串流
- 預設長度（32 bytes）時自動啟用 `blake3::hash()` 的多線程路徑

**收益**：大檔案 BLAKE3 hash 速度提升 3-8x（視 CPU 核心數）

---

## 錯誤處理策略

### 分層錯誤

| 層級 | 錯誤類型 | 處理方式 |
|---|---|---|
| 個別檔案 | `io::Error` | 記錄錯誤、繼續收集結果；命令最終回傳 exit code 2，hash 不寫出不完整檔案 |
| Hash 行解析 | `HashFileParseError` | CLI 嚴格拒絕無效記錄，回傳 exit code 3 |
| Path 驗證 | `InvalidPath` | 拒絕該路徑，記為 error |
| 整體流程 | `HashMismatch` | 累積不匹配計數，最終 exit code 1 |
| 致命錯誤 | `Io` / `Serde` | 立即退出，exit code 2 |

### Graceful Degradation

- hash 檔中格式錯誤的行 → CLI 停止驗證並回報格式錯誤；library 的寬鬆 parser 保留相容行為
- 個別檔案無法讀取 → 記錄錯誤，繼續處理其他檔案
- 目錄中無檔案 → 警告但不報錯

目錄 walker 不忽略 `walkdir` 錯誤，避免因權限或 I/O 問題漏掃檔案後仍生成成功結果。預設 checksum 檔放在來源旁，與 checksum 記錄採用的來源父目錄基準一致。

---

## 輸出契約與退出碼（2.1.0 決策紀錄）

2.1.0 把「輸出」提升為公開契約，理由與取捨如下。

### 1. stdout 只放資料、stderr 只放訊息

**決策**：所有進度、`✔ 已儲存`、統計摘要一律走 stderr。

**理由**：`oneechk hash . -o - | sha256sum -c -` 這類管線必須能直接使用。
若摘要混進 stdout，使用者就得額外過濾，而過濾規則本身就是不穩定的耦合。

**代價**：使用者看不到「摘要」除非終端機合併兩個串流（終端機預設就是合併的，所以體驗不變）。

### 2. 四段式退出碼

| 碼 | 語意 | 為什麼需要獨立 |
| :---: | :--- | :--- |
| 0 | 成功 | — |
| 1 | 驗證失敗／有差異 | 這是「預期內的不一致」，CI 要失敗但不算工具壞掉 |
| 2 | 執行或 I/O 錯誤 | 工具本身的環境問題，需要重試或修環境 |
| 3 | 用法或輸入錯誤 | 呼叫方式錯了，重試無用 |
| 4 | 無可處理項目 | 空目錄不是錯誤，但確實什麼都沒做 |

**理由**：二值退出碼無法區分「檔案不符」與「清單壞掉」，而這兩者的處置完全不同。
`4` 獨立出來，是為了讓「掃到空目錄」在 CI 中不被誤判成成功。

### 3. 格式以資料結構為中心

`report` 模組先產生 `VerifyReport`，再由 `write_verify_report` 依格式輸出。
因此 `json`、`csv`、`ndjson` 的欄位永遠一致（同一份 Rust struct 序列化），
新增欄位時所有格式同時受益。

### 4. 為何不採用 `--check` 相容旗標

`sha256sum --check` 的輸出格式與退出碼是二值的，無法承載 mismatch/missing/error 的區分。
`oneechk` 選擇獨立的 `verify` 子命令，並用 `shasum` 格式輸出保持與 coreutils 的互操作性
（可產生、也可消費 `sha256sum -c` 的清單）。
