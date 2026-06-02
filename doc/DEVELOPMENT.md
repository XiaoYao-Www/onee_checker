# 開發指南

## 環境需求

- **Rust**: 1.70+ (stable)
- **OS**: Windows 10+, Linux (kernel 4.4+), macOS 10.15+

## 快速開始

```bash
# Clone
git clone https://github.com/XiaoYao-Www/onee_checker.git
cd onee_checker

# 開發編譯
cargo check

# 執行測試
cargo test

# 本地執行
cargo run -- hash ./TEST -a sha256

# Release 編譯（含硬體加速）
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## 專案結構

```
onee_checker/
├── Cargo.toml
├── README.md
├── LICENSE
├── doc/                    # 詳細文件
│   ├── CLI.md              # CLI 指令參考
│   ├── API.md              # Library API 參考
│   ├── SECURITY.md         # 安全性設計
│   ├── ARCHITECTURE.md     # 架構文件
│   └── DEVELOPMENT.md      # 本文件
├── 功能驗證/               # 驗證報告
│   ├── 驗證規劃_*.md
│   └── 驗證結果*.md
├── src/
│   ├── lib.rs              # Library 公開 API + prelude
│   ├── main.rs             # CLI 入口
│   ├── algorithm.rs        # HashType / HashAlgo / HashData / BufferSize
│   ├── error.rs            # OneeError (thiserror)
│   ├── hash.rs             # 雜湊計算引擎
│   ├── tree.rs             # 樹狀文字輸出
│   ├── cli/                # Clap 參數定義
│   │   ├── mod.rs
│   │   ├── hash_cmd.rs
│   │   ├── verify_cmd.rs
│   │   ├── json_cmd.rs
│   │   └── txt_cmd.rs
│   ├── fs/                 # 檔案系統操作
│   │   ├── mod.rs
│   │   ├── node.rs         # FileNode / FileNodeContainer
│   │   ├── walker.rs       # walkdir 遍歷 + 深度限制 + symlink 防護
│   │   ├── writer.rs       # Hash 檔/JSON IO + HashEntry parser
│   │   └── path_safe.rs    # Path Traversal 防護
│   └── hasher/             # 雜湊器實作
│       ├── mod.rs
│       ├── enum_hasher.rs  # HasherEnum 靜態分發
│       ├── md5_hasher.rs
│       ├── sha1_hasher.rs
│       ├── sha2_hasher.rs
│       ├── sha3_hasher.rs
│       ├── blake2_hasher.rs
│       └── blake3_hasher.rs
└── TEST/                   # 測試資料 (880+ 檔案)
```

## 編譯優化

| 用途 | 指令 |
|---|---|
| Debug (快速編譯) | `cargo build` |
| Release (無加速) | `cargo build --release` |
| Release (最大效能) | `RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat" cargo build --release` |

硬體加速：

- **SHA-2**: `sha2` v0.11 自動檢測 SHA-NI（需 `target-cpu=native` 編譯）
- **BLAKE3**: 執行期自動檢測 SSE4.1 / AVX2 / AVX-512

## 測試

### 單元測試

```bash
cargo test
```

目前覆蓋：

| 模組 | 測試數 | 說明 |
|---|---|---|
| `algorithm` | 4 | BufferSize 解析, HashAlgo 轉換, 副檔名 |
| `hash` | 3 | compute_hash_reader, compute_file_hash, verify |
| `hasher::enum_hasher` | 3 | Clone 正確性, 所有演算法輸出長度 |
| `fs::writer` | 5 | HashEntry 驗證, parser 正確性, 邊界案例 |
| `fs::walker` | 3 | 檔案列表, FileNode 建構, 深度限制 |
| `fs::path_safe` | 5 | 正常路徑, dotdot 拒絕, 絕對路徑, 空路徑, 根目錄 |
| `tree` | 3 | 大小格式化 (binary/decimal/raw) |

### 整合測試

```bash
# 用 TEST/ 目錄執行完整測試矩陣
cargo run --release -- hash TEST -a sha256 -q
cargo run --release -- verify TEST.sha256 -q
```

### CI 範例

```yaml
# .github/workflows/test.yml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo build --release
```

## 程式碼風格

### 命名慣例

- **類型**: `PascalCase` — `HashType`, `FileNode`, `HasherEnum`
- **函數**: `snake_case` — `compute_file_hash`, `sanitize_rel_path`
- **常數**: `UPPER_SNAKE_CASE` — `MAX_HASH_LINES`, `MAX_DEPTH`
- **模組檔名**: `snake_case` — `path_safe.rs`, `enum_hasher.rs`

### 文件慣例

- 公開 API 使用 `///` doc comments
- 模組層級使用 `//!` module doc
- 中文與英文混用：型別/函數名用英文，註解用中文

### 錯誤處理

- Library 層: `thiserror` → `OneeError`
- CLI 層: `OneeError::exit_code()` → `std::process::exit()`
- 不應在 library 層直接 `eprintln!`（`parse_hash_file` 的 stderr 輸出為已知例外）

## 添加新演算法

1. 建立 `src/hasher/<name>_hasher.rs`
2. 實作 `update(&mut self, data: &[u8])` 和 `finish(self) -> Vec<u8>`
3. 在 `HasherEnum` 中新增變體
4. 在 `HashType::create_hasher()` 中新增 match arm
5. 在 `HashType` 枚舉中新增變體
6. 在 `HashAlgo` 中添加對應的 CLI 枚舉值
7. 更新 `HashType::suffix()`, `display_name()`, `default_length()`
8. 添加單元測試

範例（SHA-1 為例）：

```rust
// 1. src/hasher/sha1_hasher.rs
#[derive(Clone)]
pub struct Sha1Hasher(Sha1);
impl Sha1Hasher {
    pub fn new() -> Self { Self(Sha1::new()) }
    pub fn update(&mut self, data: &[u8]) { self.0.update(data); }
    pub fn finish(self) -> Vec<u8> { self.0.finalize().to_vec() }
}

// 2. src/hasher/enum_hasher.rs
pub enum HasherEnum {
    // ... existing ...
    Sha1(Sha1Hasher),
}

// 3. src/algorithm.rs
pub enum HashType {
    // ... existing ...
    SHA1,
}
impl HashType {
    pub fn create_hasher(&self) -> HasherEnum {
        match self {
            // ... existing ...
            Self::SHA1 => HasherEnum::Sha1(Sha1Hasher::new()),
        }
    }
}
```

## 提交規範

```
<type>: <簡短描述>

[可選的詳細說明]

Type:
  feat     新功能
  fix      Bug 修復
  refactor 重構
  docs     文件更新
  test     測試相關
  security 安全性修復
  perf     效能改進
```

## 相依版本策略

- **patch** (x.y.Z): 自動更新，無需審查
- **minor** (x.Y.z): 審查 changelog 後更新
- **major** (X.y.z): 評估 API 變更影響後決定
