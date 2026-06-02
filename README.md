# Onee Checker

[![Language](https://img.shields.io/badge/language-rust-red?logo=rust)](https://rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Author](https://img.shields.io/badge/author-XiaoYao--Www-blue)](https://github.com/XiaoYao-Www)
[![Trademark](https://img.shields.io/badge/trademark-retained-orange.svg)](NOTICE)
[![CI](https://github.com/XiaoYao-Www/onee_checker/actions/workflows/ci.yml/badge.svg)](https://github.com/XiaoYao-Www/onee_checker/actions/workflows/ci.yml)

> Copyright © 2026 [逍遙 (XiaoYao)](https://github.com/XiaoYao-Www).  
> SPDX-License-Identifier: MIT  

**Onee Checker** (`oneechk`) — 專業 hash 驗證與目錄結構工具。

提供 14 種密碼學雜湊演算法（MD5, SHA-1/2/3, SHAKE-128/256, BLAKE2s/b, BLAKE3），支援多線程並行計算、目錄結構 JSON/txt 輸出、以及 `shasum` 相容格式的 hash 驗證。

可作為 **CLI 工具** 由其他程式／腳本調用，也可作為 **Rust library** 嵌入其他專案。

---

## 快速開始

### 安裝

```bash
# 從原始碼編譯（建議啟用 CPU 原生指令加速）
RUSTFLAGS="-C target-cpu=native" cargo build --release

# 安裝到系統 PATH
cargo install --path .
```

### 基本使用

```bash
# 計算 SHA-256 hash（預設算法）
oneechk hash /path/to/directory

# 計算多種算法
oneechk hash /path/to/directory -a sha256 -a blake3 -a md5

# 驗證 hash 檔
oneechk verify directory.sha256

# 生成 JSON 目錄結構
oneechk json /path/to/directory

# 生成樹狀結構文字檔
oneechk txt /path/to/directory -s binary -m
```

---

## 支援的演算法

| 演算法 | CLI 參數 | 輸出長度 (bits) | 可變長度 |
|---|---|---|---|
| MD5 | `md5` | 128 | ❌ |
| SHA-1 | `sha1` | 160 | ❌ |
| SHA-224 | `sha224` | 224 | ❌ |
| SHA-256 | `sha256` | 256 | ❌ |
| SHA-384 | `sha384` | 384 | ❌ |
| SHA-512 | `sha512` | 512 | ❌ |
| SHA3-256 | `sha3-256` | 256 | ❌ |
| SHA3-512 | `sha3-512` | 512 | ❌ |
| SHAKE-128 | `shake128` | 256 (可調) | ✅ |
| SHAKE-256 | `shake256` | 512 (可調) | ✅ |
| BLAKE2s-256 | `blake2s256` | 256 | ❌ |
| BLAKE2b-512 | `blake2b512` | 512 | ❌ |
| BLAKE3 | `blake3` | 256 (可調) | ✅ |

---

## 命令參考

### `oneechk hash` — 生成 hash 驗證檔

```
oneechk hash [OPTIONS] <PATH>

Arguments:
  <PATH>              要計算的路徑（檔案或目錄）

Options:
  -a, --algo <ALGO>   演算法（可多次指定）[預設: sha256]
  -o, --output <OUT>  輸出檔案路徑。指定 `-` 輸出到 stdout
                       [預設: <dirname>.<算法副檔名>]
      --length <LEN>   可變長度演算法的輸出長度 (bytes)
      --buffer <SIZE>  讀取緩衝區大小，支援 K/M/G 後綴 [預設: 1M]
  -t, --threads <N>    並行線程數 [預設: CPU 邏輯核心數]
  -q, --quiet          安靜模式，不顯示進度條
```

**範例：**

```bash
# 預設 SHA-256
oneechk hash ./my-files

# 指定 BLAKE3 且自訂輸出長度 64 bytes
oneechk hash ./my-files -a blake3 --length 64

# 同時計算三種算法
oneechk hash ./my-files -a sha256 -a md5 -a blake3

# 輸出 hash 到 stdout（供其他程式解析）
oneechk hash ./my-files -o - -q

# 自訂 4K buffer 與 2 線程
oneechk hash ./my-files --buffer 4K -t 2
```

### `oneechk verify` — 驗證 hash 驗證檔

```
oneechk verify [OPTIONS] <HASHFILE>

Arguments:
  <HASHFILE>          hash 驗證檔路徑

Options:
  -a, --algo <ALGO>   手動指定演算法 [預設: 從副檔名推斷]
  -r, --root <DIR>    根目錄路徑 [預設: hash 檔所在目錄]
      --buffer <SIZE>  讀取緩衝區大小 [預設: 1M]
  -t, --threads <N>    並行線程數
  -q, --quiet          僅顯示最終摘要
```

**範例：**

```bash
# 驗證（演算法從副檔名自動推斷）
oneechk verify files.sha256

# 手動指定演算法
oneechk verify checksums -a md5

# 指定根目錄（hash 檔中的相對路徑以此為基準）
oneechk verify files.sha256 -r /backup/location
```

### `oneechk json` — 生成 JSON 目錄結構

```
oneechk json [OPTIONS] <PATH>

Arguments:
  <PATH>              要掃描的目錄

Options:
  -o, --output <OUT>  輸出檔案路徑。指定 `-` 輸出到 stdout
                       [預設: <dirname>.struct.json]
  -q, --quiet          安靜模式
```

**輸出格式：**

```json
{
  "version": "0.1.0",
  "generation_time": 1717200000,
  "nodes": [
    {
      "name": "my-files",
      "is_dir": true,
      "is_symlink": false,
      "extension": null,
      "size": 1048576,
      "last_modified": 1717200000,
      "created_at": 1717200000,
      "children": [
        {
          "name": "photo.jpg",
          "is_dir": false,
          "extension": "jpg",
          "size": 524288,
          ...
        }
      ]
    }
  ]
}
```

### `oneechk txt` — 生成樹狀目錄結構

```
oneechk txt [OPTIONS] <PATH>

Arguments:
  <PATH>              要掃描的目錄

Options:
  -o, --output <OUT>  輸出檔案路徑。指定 `-` 輸出到 stdout
                       [預設: <dirname>.tree.txt]
  -s, --size <TYPE>   大小顯示格式 [binary, decimal, raw]
  -m, --modified      顯示最後修改時間
  -c, --created       顯示建立時間
  -q, --quiet         安靜模式
```

**輸出範例：**

```
# ******************************
# Generation Time: 2026-06-02 15:30:00 +0800
# ******************************

my-files (1.00 GiB) [mod: 2026-01-15 10:30:00 +08:00]
├── photos (512.00 MiB) [mod: 2026-01-10 08:00:00 +08:00]
│   ├── vacation.jpg (2.35 MiB) [mod: 2025-12-20 14:22:10 +08:00]
│   └── family.jpg (3.12 MiB) [mod: 2025-12-21 09:15:45 +08:00]
└── documents (256.00 MiB) [mod: 2026-01-14 16:45:30 +08:00]
    ├── report.pdf (1.50 MiB) [mod: 2026-01-14 10:00:00 +08:00]
    └── notes.txt (12.50 KiB) [mod: 2026-01-13 22:30:15 +08:00]
```

---

## 作為底層基礎設施調用

Onee Checker 設計為可被其他軟體／腳本以 subprocess 方式調用。

### stdout/stderr 分離原則

- **stdout** → 機器可解析的資料（hash 行、JSON、樹狀文字）
- **stderr** → 人類可讀的訊息（進度、錯誤、狀態）

### Exit Code

| Code | 含義 |
|---|---|
| **0** | 成功 |
| **1** | hash 不匹配（驗證失敗） |
| **2** | I/O 或內部錯誤 |
| **3** | 使用者輸入錯誤 |

### 典型調用範例

```bash
# 生成 hash 並 pipe 給其他工具
oneechk hash /data -o - -q | grep "important_file"

# 驗證並從 exit code 判斷結果
oneechk verify /data/checksums.sha256 -q
if [ $? -eq 0 ]; then
    echo "完整性驗證通過"
else
    echo "檔案損壞或已變更！"
fi

# JSON 輸出給 jq 解析
oneechk json /data -o - -q | jq '.nodes[0].children[] | select(.size > 1048576)'
```

---

## 作為 Rust Library 使用

```rust
use onee_checker::prelude::*;
use std::path::Path;

fn main() -> Result<()> {
    // 並行計算整個目錄的 SHA-256
    let files = list_files(Path::new("/data"))?;
    let results = compute_hashes_parallel(&files, &HashType::SHA256, 1024 * 1024);

    for result in results {
        match result {
            Ok(data) => println!("{} = {}", data.path.display(), data.hash_hex()),
            Err(e) => eprintln!("Error: {e}"),
        }
    }

    // 驗證 hash 檔
    let verify_results = verify_hash_file(
        Path::new("checksums.sha256"),
        &HashType::SHA256,
        Path::new("/data"),
        1024 * 1024,
    );

    Ok(())
}
```

### Library 公開 API 模組

| 模組 | 功能 |
|---|---|
| `algorithm` | `HashType`, `HashAlgo`, `HashData`, `BufferSize` |
| `hash` | `compute_file_hash`, `compute_hashes_parallel`, `verify_hash_file` |
| `hasher` | `HasherEnum` — 零 heap alloc 的 14 種雜湊器枚舉 |
| `fs` | `list_files`, `build_file_node`, `parse_hash_file`, `sanitize_rel_path` |
| `tree` | `write_tree`, `SizeFormat`, `TreeOption` |
| `error` | `OneeError` — 結構化錯誤，含 exit code |

---

## 安全性

Onee Checker 在設計中考慮了以下安全防護：

| 防護 | 機制 |
|---|---|
| **Path Traversal** | `sanitize_rel_path()` 拒絕 `..`、絕對路徑、null byte |
| **Symlink 逃脫** | `build_node_recursive` 驗證 symlink 目標在根目錄內 |
| **Hash Parser DoS** | 行數上限 10M、hex 格式驗證 |
| **Windows 裝置名稱** | 拒絕 CON、NUL、PRN 等保留名稱 |
| **Buffer overflow** | `u16` 限制可變長度雜湊長度最大值 |
| **Stack overflow** | `build_node_recursive` 深度上限 1024 層 |

詳見 `功能驗證/` 目錄下的完整驗證報告。

---

## 效能

### 硬體加速

- **SHA-2**: 在 x86_64 平台自動啟用 SHA-NI 指令（需以 `RUSTFLAGS="-C target-cpu=native"` 編譯）
- **BLAKE3**: 執行期自動檢測 SSE4.1 / AVX2 / AVX-512
- **HasherEnum**: 零 heap alloc 的 enum 靜態分發，編譯器可完全 inline

### 建議編譯參數

```bash
# 最大效能
RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat" cargo build --release
```

---

## 專案結構

```
src/
├── lib.rs              # Library 公開 API + prelude
├── main.rs             # CLI 薄調度層
├── algorithm.rs        # HashType / HashAlgo / HashData
├── error.rs            # OneeError (thiserror)
├── hash.rs             # 雜湊計算引擎
├── tree.rs             # 樹狀文字輸出
├── cli/                # Clap 參數定義（4 子命令）
├── fs/                 # 檔案系統操作 + 安全防護
│   ├── node.rs         # FileNode 結構
│   ├── walker.rs       # walkdir 遍歷
│   ├── writer.rs       # Hash 檔/JSON 寫入 + parser
│   └── path_safe.rs    # Path traversal 防護
└── hasher/             # 14 種雜湊演算法實作
    ├── mod.rs
    ├── enum_hasher.rs  # HasherEnum 零成本靜態分發
    ├── md5_hasher.rs
    ├── sha1_hasher.rs
    ├── sha2_hasher.rs
    ├── sha3_hasher.rs
    ├── blake2_hasher.rs
    └── blake3_hasher.rs
```

---

## 相依套件

| 依賴 | 用途 | 版本 |
|---|---|---|
| `clap` | CLI 參數解析 | 4.6 |
| `rayon` | 並行計算 | 1.12 |
| `walkdir` | 目錄遍歷 | 2.5 |
| `sha2` | SHA-2 系列演算法 | 0.11 |
| `sha3` | SHA-3 / SHAKE | 0.11 |
| `blake3` | BLAKE3 | 1.8 |
| `indicatif` | 進度條 | 0.18 |
| `serde` / `serde_json` | JSON 序列化 | 1.0 |
| `thiserror` | 結構化錯誤 | 2 |
| `chrono` | 時間格式化 | 0.4 |

---

## 效能基準

在 x86_64 處理器上以 `cargo bench` 測得（release 模式）：

| 情境 | 吞吐量 | 備註 |
|------|--------|------|
| SHA-256 1MB | ~2.6 GB/s | `sha2` crate，含 SHA-NI 加速 |
| BLAKE3 1MB | ~5.7 GB/s | 自動偵測 AVX2 |
| BLAKE3 100MB | ~5.1 GB/s | 串流模式，O(1) 記憶體 |
| hex 編碼 (32 bytes) | ~310 MB/s | `hex::encode` |

> 完整基準報告：執行 `cargo bench`

---

## License

本專案以 [MIT License](LICENSE) 授權。

**商標保留**: "Onee Checker"、"oneechk" 名稱及專案標誌為作者商標。MIT 授權不包含商標使用權。衍生作品或分支必須更名以避免混淆。詳見 [NOTICE](NOTICE)。

Copyright © 2026 逍遙 (XiaoYao) — [GitHub](https://github.com/XiaoYao-Www)

依賴套件的授權詳見 [doc/LICENSE-AUDIT.md](doc/LICENSE-AUDIT.md)。
