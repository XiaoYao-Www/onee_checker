# Library API 參考

Onee Checker 可作為 Rust library (`onee_checker`) 嵌入其他專案。公開 API 透過 `onee_checker::prelude::*` 匯出。

> 本文對應版本 **2.1.0**。2.1.0 新增 `report`（輸出契約）與 `manifest`（清單解析／比對）兩個模組，
> 並在 `OneeError` 加入 `NothingToDo`（CLI 退出碼 4）。

---

## 模組概覽

```
onee_checker
├── algorithm    — 核心類型：HashType, HashAlgo, HashData, BufferSize
├── hash         — 雜湊計算引擎
├── hasher       — 14 種演算法實作 + HasherEnum 靜態分發
├── fs           — 檔案系統操作與安全防護
├── tree         — 樹狀目錄結構輸出
├── report       — 輸出格式與驗證報告（v2.1.0 新增）
├── manifest     — 清單格式偵測／解析／diff（v2.1.0 新增）
├── error        — 結構化錯誤 (OneeError)
└── cli          — Clap 參數定義 (供 CLI 使用，embedder 可忽略)
```

---

## `onee_checker::algorithm`

### `HashType`

```rust
pub enum HashType {
    MD5,
    SHA1,
    SHA224, SHA256, SHA384, SHA512,
    SHA3_256, SHA3_512,
    SHAKE128(u16), SHAKE256(u16),
    BLAKE2S256, BLAKE2B512,
    BLAKE3(u16),
}
```

| 方法 | 回傳 | 說明 |
|---|---|---|
| `create_hasher()` | `HasherEnum` | 建立對應 hasher (零 heap alloc) |
| `can_specify_length()` | `bool` | 是否支援自訂輸出長度 |
| `default_length()` | `u16` | 預設輸出長度 (bytes) |
| `suffix()` | `String` | 副檔名 (如 `"sha256"`) |
| `display_name()` | `&'static str` | 人類可讀名稱 |

### `HashAlgo`

```rust
pub enum HashAlgo {
    Md5, Sha1, Sha224, Sha256, Sha384, Sha512,
    Sha3256, Sha3512,
    Shake128, Shake256,
    Blake2s256, Blake2b512,
    Blake3,
}
```

| 方法 | 回傳 | 說明 |
|---|---|---|
| `to_hash_type(length: Option<u16>)` | `Result<HashType>` | 轉換為 HashType |
| `from_suffix(suffix: &str)` | `Option<Self>` | 從副檔名推斷 |
| `can_specify_length()` | `bool` | |
| `default_length()` | `u16` | |

### `HashData`

```rust
pub struct HashData {
    pub path: PathBuf,
    pub hash_bytes: Vec<u8>,
    pub hash_type: HashType,
}
```

| 方法 | 說明 |
|---|---|
| `new(path, bytes, hash_type)` | 建構子 |
| `hash_hex()` | 回傳 hex 字串 (分配新 String) |
| `hash_hex_into(&mut String)` | 將 hex 寫入外部緩衝區 (零 alloc) |

### `BufferSize`

```rust
pub struct BufferSize(pub usize);
```

實作 `FromStr`，支援人類可讀後綴 (`4K`, `1M`, `2G`)。

---

## `onee_checker::hash`

### `compute_hash_reader`

```rust
pub fn compute_hash_reader<R: Read>(
    reader: R,
    hasher: &mut HasherEnum,
    buffer: &mut [u8],
) -> io::Result<Vec<u8>>
```

Streaming hash 計算。`buffer` 為外部提供的可復用緩衝區。

### `compute_file_hash`

```rust
pub fn compute_file_hash(
    path: &Path,
    hash_type: &HashType,
    buffer: &mut [u8],
) -> io::Result<Vec<u8>>
```

計算單一檔案的 hash。拒絕目錄路徑。

### `compute_hashes_parallel`

```rust
pub fn compute_hashes_parallel(
    files: &[PathBuf],
    hash_type: &HashType,
    buffer_size: usize,
) -> Vec<Result<HashData, OneeError>>
```

使用 rayon 並行計算多個檔案。`buffer_size` 為每個 worker 的緩衝區大小。

> 每個 worker 會在 `map_init` 中 clone `HasherEnum` template，無逐檔 heap alloc。

### `verify_file_hash`

```rust
pub fn verify_file_hash(
    path: &Path,
    expected_hex: &str,
    hash_type: &HashType,
    buffer: &mut [u8],
) -> Result<bool, OneeError>
```

驗證單一檔案 hash 是否匹配預期值。

### `compute_multi_hashes_parallel`（v2.0.1 新增）

```rust
pub fn compute_multi_hashes_parallel(
    files: &[PathBuf],
    hash_types: &[HashType],
    buffer_size: usize,
) -> Vec<Vec<Result<HashData, OneeError>>>
```

**單次 I/O 多演算法並行計算**。讀取每個檔案一次，同時更新多個 hasher。

- **輸入**：檔案列表、演算法列表（2+）、buffer 大小
- **輸出**：`Vec<Vec<Result<...>>>` — 外層 per-algorithm，內層 per-file
- 當 `hash_types.len() == 1` 時等同 `compute_hashes_parallel`

**使用時機**：需要同時計算多種 hash 時，取代逐個呼叫 `compute_hashes_parallel`。

```rust
use onee_checker::prelude::*;

let files = list_files(Path::new("/data"))?;
let results = compute_multi_hashes_parallel(
    &files,
    &[HashType::SHA256, HashType::BLAKE3(32)],
    1024 * 1024,
);

// results[0] = SHA256 結果
// results[1] = BLAKE3 結果
for result in &results[0] {
    if let Ok(data) = result {
        println!("SHA256: {} = {}", data.path.display(), data.hash_hex());
    }
}
```

### `compute_multi_hashes_parallel_with_pool`

與 `compute_multi_hashes_parallel` 相同，但使用指定的 `rayon::ThreadPool`。

```rust
pub fn compute_multi_hashes_parallel_with_pool(
    pool: &ThreadPool,
    files: &[PathBuf],
    hash_types: &[HashType],
    buffer_size: usize,
) -> Vec<Vec<Result<HashData, OneeError>>>
```

### `verify_hash_file`

```rust
pub fn verify_hash_file(
    hash_file_path: &Path,
    hash_type: &HashType,
    root_dir: &Path,
    buffer_size: usize,
) -> Vec<Result<(PathBuf, bool, String), OneeError>>
```

驗證整個 hash 檔。回傳每筆記錄的結果：

- `(path, is_match, actual_hex)` on success
- `Err(OneeError)` on path validation failure

> **安全**: 內部會 `canonicalize_root` + `sanitize_rel_path` 防止 path traversal。

---

## `onee_checker::hasher`

### `HasherEnum`

```rust
pub enum HasherEnum {
    Md5(Md5Hasher),
    Sha1(Sha1Hasher),
    Sha224(Sha224Hasher),
    // ... 共 14 變體
}
```

| 方法 | 說明 |
|---|---|
| `update(&mut self, data: &[u8])` | 餵入資料 |
| `finalize(self) -> Vec<u8>` | 完成計算，回傳 hash bytes |
| `Clone` | 複製一份獨立 hasher（約 64-200 bytes） |

> 零 heap alloc：所有變體存於 stack。`update()` / `finalize()` 為靜態分發，編譯器可完全 inline。

### 個別 Hasher 類型

```rust
Md5Hasher        Sha1Hasher       Sha224Hasher
Sha256Hasher     Sha384Hasher     Sha512Hasher
Sha3_256Hasher   Sha3_512Hasher   Shake128XofHasher
Shake256XofHasher Blake2s256Hasher Blake2b512Hasher
Blake3Hasher
```

所有類型均實作 `Clone`、`Send`、`Sync`。每個類型提供 `new()`、`update(&mut self, data: &[u8])`、`finish(self) -> Vec<u8>`。

### `blake3_hash_bulk`（v2.0.1 新增）

```rust
pub fn blake3_hash_bulk(data: &[u8], out_len: u16) -> Vec<u8>
```

對整個 buffer 計算 BLAKE3 hash。當 `out_len == 32`（預設長度）時使用 `blake3::hash()` 內部多線程樹狀 hash。
自訂長度時使用串流模式。

```rust
use onee_checker::hasher::blake3_hash_bulk;

// 多線程：32 bytes 輸出，使用 blake3::hash() 內部並行
let hash = blake3_hash_bulk(&large_file_data, 32);

// 串流：64 bytes 輸出，走 update() + finalize_xof()
let hash = blake3_hash_bulk(&data, 64);
```

---

## `onee_checker::fs`

### 目錄遍歷

```rust
pub fn list_files(path: &Path) -> io::Result<Vec<PathBuf>>
pub fn build_file_node(path: &Path) -> io::Result<FileNode>
pub fn validate_path(path: &Path) -> Result<(), String>
```

### Hash 檔讀寫

```rust
pub fn save_hash_file(data: &[HashData], output: &Path, root: &Path) -> io::Result<()>
pub fn save_file_node_json(container: &FileNodeContainer, output: &Path) -> io::Result<()>
pub fn parse_hash_file(content: &str) -> Vec<HashEntry>
pub fn parse_hash_file_strict(content: &str) -> Result<Vec<HashEntry>>
```

`parse_hash_file` 為相容用途保留寬鬆解析行為；嚴格版本遇到無效記錄、超過行數上限或空清單時回傳 `HashFileParseError`。CLI 與驗證 API 採嚴格解析。

### 安全防護

```rust
pub fn canonicalize_root(root: &Path) -> Result<PathBuf>
pub fn sanitize_rel_path(rel_path: &str, root: &Path) -> Result<PathBuf>
```

詳見 [SECURITY.md](SECURITY.md)。

### 資料結構

```rust
pub struct FileNode {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub extension: Option<String>,
    pub size: u64,
    pub last_modified: Option<i64>,
    pub created_at: Option<i64>,
    pub children: Option<Vec<FileNode>>,
    pub symlink_target: Option<String>,
}

pub struct FileNodeContainer {
    pub version: String,
    pub generation_time: i64,
    pub nodes: Vec<FileNode>,
}

pub struct HashEntry {
    pub hash_hex: String,
    pub rel_path: String,
}
```

### 工具函數

```rust
pub fn to_unix_timestamp(time: SystemTime) -> i64
```

---

## `onee_checker::tree`

```rust
pub enum SizeFormat { Binary, Decimal, Raw }

pub struct TreeOption {
    pub size: Option<SizeFormat>,
    pub last_modified: bool,
    pub created_at: bool,
}

pub fn write_tree<W: Write>(writer: &mut W, path: &Path, option: &TreeOption) -> io::Result<()>
pub fn format_unix_to_local(secs: i64) -> String
```

---

## `onee_checker::error`

### `OneeError`

```rust
pub enum OneeError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    HashMismatch { path: PathBuf, expected: String, actual: String },
    HashFileParseError { line: usize, detail: String },
    InvalidPath(String),
    UnsupportedLength { algorithm: String },
    UnsupportedAlgorithm(String),
    ArgumentError(String),
    NothingToDo,
}
```

| 變體 | CLI 退出碼 |
| :--- | :---: |
| `HashMismatch` | 1 |
| `Io`、`Serde` | 2 |
| `HashFileParseError`、`InvalidPath`、`UnsupportedLength`、`UnsupportedAlgorithm`、`ArgumentError` | 3 |
| `NothingToDo` | 4 |

| 方法 | 說明 |
|---|---|
| `exit_code() -> i32` | 專案 CLI 契約：0 成功、1 hash 不匹配、2 I/O/執行錯誤、3 輸入/格式錯誤、4 無可處理項目 |

### `Result<T>`

```rust
pub type Result<T> = std::result::Result<T, OneeError>;
```

---

## `onee_checker::report`（v2.1.0 新增）

輸出契約層：把「要輸出什麼」與「怎麼寫」分開，讓 CLI 與 embedder 共用同一套格式。

### `OutputFormat`

```rust
pub enum OutputFormat { Human, Shasum, Bsd, Json, Csv, Ndjson }
```

`clap::ValueEnum`，`Default` 為 `Human`，CLI 上即 `--format` 的合法值。

### `EntryStatus`

```rust
pub enum EntryStatus { Match, Mismatch, Error, Missing }

impl EntryStatus {
    pub fn as_str(&self) -> &'static str;   // "match" / "mismatch" / "error" / "missing"
}
```

### `EntryReport`

```rust
pub struct EntryReport {
    pub path: String,
    pub status: EntryStatus,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub error: Option<String>,
}

impl EntryReport {
    pub fn new(path: impl Into<String>, status: EntryStatus) -> Self;
}
```

路徑建議使用相對於根目錄、以 `/` 分隔的形式，這樣輸出可以跨平台一致。

### `VerifySummary` 與 `VerifyReport`

```rust
pub struct VerifySummary { pub total: u64, pub matched: u64, pub mismatched: u64,
                           pub errors: u64, pub missing: u64 }

impl VerifySummary {
    pub fn from_entries(entries: &[EntryReport]) -> Self;
    pub fn add(&mut self, status: EntryStatus);
}

pub struct VerifyReport {
    pub version: String, pub command: String, pub hashfile: String,
    pub algorithm: String, pub root: String, pub summary: VerifySummary,
    pub exit_code: i32, pub entries: Vec<EntryReport>,
}

impl VerifyReport {
    pub const fn compute_exit_code(&self) -> i32;
}
```

`compute_exit_code()` 的優先序：`errors > 0 → 2`、`mismatched > 0 → 1`、`total == 0 → 4`、否則 `0`。

### 寫入函式

```rust
pub fn csv_escape(field: &str) -> String;

pub fn write_hash_line<W: Write>(w: &mut W, format: OutputFormat,
    algorithm: &str, hash_hex: &str, rel_path: &str) -> io::Result<()>;

pub fn write_entry<W: Write>(w: &mut W, format: OutputFormat,
    entry: &EntryReport) -> io::Result<()>;

pub fn write_verify_report<W: Write>(w: &mut W, format: OutputFormat,
    report: &VerifyReport) -> io::Result<()>;
```

`Human` 格式在 `write_*` 中不輸出資料（人類訊息由呼叫端寫到 stderr），
所以把 `Human` 傳進來是安全的 no-op。

### 範例：產生 JSON 驗證報告

```rust
use onee_checker::report::{EntryReport, EntryStatus, VerifyReport, VerifySummary};

let entries = vec![
    EntryReport::new("dist/a.txt", EntryStatus::Match),
];

let report = VerifyReport {
    version: env!("CARGO_PKG_VERSION").to_string(),
    command: "verify".to_string(),
    hashfile: "dist.sha256".to_string(),
    algorithm: "SHA-256".to_string(),
    root: ".".to_string(),
    summary: VerifySummary::from_entries(&entries),
    exit_code: 0,
    entries,
};

let mut out = std::io::stdout();
onee_checker::report::write_verify_report(&mut out, onee_checker::report::OutputFormat::Json, &report)?;
# Ok::<(), std::io::Error>(())
```

---

## `onee_checker::manifest`（v2.1.0 新增）

清單（manifest）解析與比對：支援 Shasum、BSD、CSV、JSON 四種格式。

### `ManifestFormat`

```rust
pub enum ManifestFormat { Shasum, Bsd, Csv, Json }

impl ManifestFormat {
    pub const fn as_str(self) -> &'static str;
    pub fn from_suffix(suffix: &str) -> Option<Self>;   // ".sha256" / "csv" / "json" …
}
```

### 偵測與解析

```rust
pub fn detect_format(content: &str) -> Option<ManifestFormat>;
pub fn parse_auto(content: &str) -> Result<(ManifestFormat, Vec<HashEntry>)>;
pub fn parse(content: &str, format: ManifestFormat) -> Result<Vec<HashEntry>>;
```

`detect_format` 的優先序為 **BSD > JSON > CSV > Shasum**；`parse_auto` 在偵測到的格式解析失敗時
會退回 Shasum 再試一次，因此把 `sha256sum` 產生的檔案或含逗號路徑的清單丟進來都能解析。

`HashEntry { hash_hex, rel_path }` 由 `onee_checker::fs` 提供；`rel_path` 保留原始分隔符。

### `DiffReport`

```rust
pub struct ChangedEntry { pub path: String, pub old_hash: String, pub new_hash: String }

pub struct DiffReport {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<ChangedEntry>,
    pub unchanged: u64,
}

impl DiffReport {
    pub fn is_empty(&self) -> bool;
    pub fn total_changes(&self) -> usize;
}

pub fn diff(old: &[HashEntry], new: &[HashEntry]) -> DiffReport;
```

`diff` 以 `rel_path` 為鍵、十六進位比較不分大小寫，結果依路徑升冪排序；
`DiffReport` 與 `ChangedEntry` 都實作 `serde::Serialize`，可直接輸出 JSON。

### 範例：比對兩份清單

```rust
use onee_checker::manifest::{diff, parse_auto};

let (_, v1) = parse_auto(&std::fs::read_to_string("v1.sha256")?)?;
let (_, v2) = parse_auto(&std::fs::read_to_string("v2.sha256")?)?;

let report = diff(&v1, &v2);
println!("{} 新增, {} 移除, {} 變更",
    report.added.len(), report.removed.len(), report.changed.len());
# Ok::<(), onee_checker::error::OneeError>(())
```

---

## 完整範例

```rust
use onee_checker::prelude::*;
use std::path::Path;

fn integrity_check(data_dir: &Path) -> Result<()> {
    // 1. 計算所有檔案 SHA-256
    let files = list_files(data_dir)?;
    let results = compute_hashes_parallel(&files, &HashType::SHA256, 1024 * 1024);

    // 2. 儲存 hash 檔
    let hash_data: Vec<HashData> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    let output = data_dir.join("checksums.sha256");
    save_hash_file(&hash_data, &output, data_dir)?;

    // 3. 產生目錄快照 JSON
    let node = build_file_node(data_dir)?;
    let container = FileNodeContainer {
        version: "0.1.0".into(),
        generation_time: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        nodes: vec![node],
    };
    save_file_node_json(&container, &data_dir.join("snapshot.json"))?;

    Ok(())
}
```
