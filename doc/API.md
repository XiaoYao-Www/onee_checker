# Library API 參考

Onee Checker 可作為 Rust library (`onee_checker`) 嵌入其他專案。公開 API 透過 `onee_checker::prelude::*` 匯出。

---

## 模組概覽

```
onee_checker
├── algorithm    — 核心類型：HashType, HashAlgo, HashData, BufferSize
├── hash         — 雜湊計算引擎
├── hasher       — 14 種演算法實作 + HasherEnum 靜態分發
├── fs           — 檔案系統操作與安全防護
├── tree         — 樹狀目錄結構輸出
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
```

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
}
```

| 方法 | 說明 |
|---|---|
| `exit_code() -> i32` | 0/1/2/3 對應 POSIX exit code |

### `Result<T>`

```rust
pub type Result<T> = std::result::Result<T, OneeError>;
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
