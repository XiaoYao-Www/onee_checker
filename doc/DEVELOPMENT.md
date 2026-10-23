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

## 常用指令

```bash
# 執行所有測試
cargo test --locked

# 執行基準測試
cargo bench

# 格式化檢查
cargo fmt --check

# Clippy 靜態分析（含測試與基準）
cargo clippy --all-targets --locked -- -D warnings
```

## 安全審計

```bash
# 安裝 cargo-audit
cargo install cargo-audit

# 檢查已知漏洞
cargo audit
```

## 授權合規

```bash
# 安裝 cargo-deny
cargo install cargo-deny

# 檢查授權衝突
cargo deny check
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

### Windows GNU target（選用）

本專案**不提交** Cargo 設定檔。CI 直接使用 windows-latest 內建的 MinGW 連結 `x86_64-pc-windows-gnu`；若本機想重現發布產物，建議用環境變數指定 target：

```powershell
$env:CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu"
cargo build --release --locked
```

`.cargo/config.toml` 屬於本機設定；**不要**把 `[build] target` 提交進儲存庫，否則 Linux/macOS 的 CI 也會被強迫交叉編譯 Windows。舊版放在 `cargo/config.toml` 的設定 Cargo 不會讀取（正確位置是 `.cargo/config.toml`），已於 2.0.3 移除。

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
# CLI 整合測試（23 項，包含所有演算法 + 邊界條件）
cargo test --test cli_integration

# 執行全部測試（單元 + 整合 + 文檔）
cargo test

# 用 TEST/ 目錄執行完整測試矩陣
cargo run --release -- hash TEST -a sha256 -q
cargo run --release -- verify TEST.sha256 -q
```

### 標準化驗證流程

專案提供可重複執行的驗證腳本，涵蓋編譯、測試、多線程一致性、邊界條件與安全性檢查：

```bash
# 完整驗證（含基準測試）
bash verify.sh full

# 快速驗證（跳過基準測試）
bash verify.sh quick
```

### 測試覆蓋範圍

| 層級 | 數量 | 說明 |
|---|---|---|
| 單元測試 | 37 | 各模組函數正確性 |
| 整合測試 | 23 | CLI 端到端（所有演算法、多演算法、驗證、邊界條件） |
| 基準測試 | 7 | 演算法效能、多演算法、BLAKE3 多線程比較 |

### 新增測試

整合測試位於 `tests/cli_integration.rs`，使用 `assert_cmd` 執行 CLI 二進位檔。
新增測試時請依循命名慣例：`test_cli_<子命令>_<測試內容>`。

### CI 流程

實際流程定義於 `.github/workflows/`；文件與設定不一致時，一律以 workflow 檔案為準：

| Workflow | 觸發條件 | 內容 |
|---|---|---|
| `ci.yml` | 所有分支 push、PR 到 `main` | 三平台（ubuntu/windows/macos）`cargo test --locked`、`cargo clippy --all-targets -- -D warnings`、`cargo fmt --check`；另有 `cargo audit` 與 `cargo deny check` |
| `auto_build_release.yml` | push 到 `main`、手動觸發 | 讀取 `Cargo.toml` 版本；若 `v<版本>_Auto` 標籤不存在，建置 Windows（x86_64-pc-windows-gnu）與兩個 Linux musl（x86_64/aarch64）產物並建立 GitHub Release |

推送 `main` 後會自動發布新版本。版本號以 `Cargo.toml` 的 `[package].version` 為準；發布前請同步更新 `Cargo.toml`、`Cargo.lock` 與 `doc/CLI.md` 的版本標註。

### 本機重現 CI

```bash
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo audit
cargo deny check
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

- Library 層: `thiserror` → `OneeError`；walker 回報遍歷錯誤，CLI 專用嚴格 hash parser 拒絕損壞記錄
- CLI 層: `OneeError::exit_code()` → `std::process::exit()`；一般參數錯誤為 3，I/O 為 2，hash 不匹配為 1
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
