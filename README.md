# Onee Checker

[![Language](https://img.shields.io/badge/language-rust-red?logo=rust)](https://rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/XiaoYao-Www/onee_checker/actions/workflows/ci.yml/badge.svg)](https://github.com/XiaoYao-Www/onee_checker/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.87-blue)](Cargo.toml)

**Onee Checker**（`oneechk`）是一套可腳本化、輸出可解析的檔案完整性工具：
一次 I/O 同時計算 14 種雜湊、驗證清單、比對兩份清單，並能輸出目錄結構。

與 `sha256sum` 的差別不只是「多幾種演算法」，而是**把驗證流程當成 CI 的一等公民**：
stdout 只放資料、stderr 只放訊息、退出碼細分四類，讓流水線不必解析人類文字。

---

## 目錄

- [安裝](#安裝)
- [5 分鐘上手](#5-分鐘上手)
- [常見工作流程](#常見工作流程)
- [命令速查](#命令速查)
- [輸出格式與退出碼](#輸出格式與退出碼)
- [效能設計](#效能設計)
- [作為 Rust Library 使用](#作為-rust-library-使用)
- [支援的演算法](#支援的演算法)
- [文件索引](#文件索引)
- [授權](#授權)

## 安裝

### 預編譯執行檔（GitHub Releases）

每次 `main` 的版本號變更會自動建置三個平台，檔名格式為
`oneechk-<版本>-<target>.zip`（內含版本化執行檔、README、LICENSE、NOTICE）：

| 平台 | target |
| :--- | :--- |
| Windows x64 | `x86_64-pc-windows-gnu` |
| Linux x64（musl） | `x86_64-unknown-linux-musl` |
| Linux arm64（musl） | `aarch64-unknown-linux-musl` |

解壓後把 `oneechk.exe`（或 `oneechk`）放進 `PATH` 即可，無需安裝執行期。

### 從原始碼

```bash
git clone https://github.com/XiaoYao-Www/onee_checker
cd onee_checker
cargo build --release          # target/release/oneechk
```

### 以 Cargo 安裝

```bash
cargo install --path .
```

需求：Rust 1.87 以上（MSRV），edition 2021。

## 5 分鐘上手

### 第 1 步：產生第一份清單

```bash
oneechk hash ./dist
```

```text
✔ 已儲存: ./dist.sha256
```

預設檔名是 `<目錄名稱>.<演算法>`，預設演算法 `sha256`。
想直接看結果、不寫檔：

```bash
oneechk hash ./dist -o -
```

```text
8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8  dist/a.txt
f44e64e75f3948e9f73f8dfa94721c4ce8cbb4f265c4790c702b2d41cfbf2753  dist/b.txt
```

### 第 2 步：驗證

```bash
oneechk verify dist.sha256
```

```text
✔ 驗證完成: 2 個檔案, 2 匹配, 0 不匹配, 0 錯誤
```

把檔案改一個位元再跑一次，退出碼會從 `0` 變成 `1`：

```bash
oneechk verify dist.sha256 -q; echo $?
# 1
```

### 第 3 步：放進腳本 / CI

```bash
oneechk verify dist.sha256 -q --format json > verify.json || exit 1
```

`--format json` 的輸出是穩定的機器契約（見 [doc/CLI.md](doc/CLI.md#輸出格式詳解)），
`stderr` 不會混進 stdout，可以直接 `jq`。

## 常見工作流程

### 發佈物校驗（下載端）

```bash
# 發佈者
oneechk hash ./artifacts -a sha256 -a blake3

# 使用者
oneechk verify artifacts.sha256 -r ./artifacts
```

### 單一檔案快速確認

```bash
oneechk check ./download.zip --expect 8ed3f6ad... --ignore-case
```

### 版本之間有什麼變了

```bash
oneechk diff v1.sha256 v2.sha256 --format json
```

```json
{"added":["data/c.txt"],"removed":[],"changed":[],"unchanged":2}
```

### 目錄盤點（給 issue 或審查用）

```bash
oneechk txt ./src -s binary -m
oneechk json ./src -o src.struct.json
```

### 多演算法一次算完

```bash
oneechk hash ./big.iso -a sha256 -a blake3 -a sha512 --digest-only
```

單次讀取 I/O，三種雜湊一起算完；檔案大時差距可達數倍。

## 命令速查

| 命令 | 用途 | 常用參數 |
| :--- | :--- | :--- |
| `hash <PATH>` | 產生清單 | `-a`（可重複）、`-o -`（stdout）、`--digest-only`、`-t` |
| `verify <HASHFILE>` | 依清單驗證 | `-r`、`--format json`、`--report`、`--ignore-missing` |
| `check <PATH>` | 單檔／文字驗證 | `--expect`、`-a`、`--ignore-case` |
| `diff <OLD> <NEW>` | 比對兩份清單 | `--format human\|csv\|json` |
| `json <PATH>` | 目錄結構 JSON | `-o` |
| `txt <PATH>` | 樹狀文字 | `-s`、`-m`、`-c` |

全域選項：`--format`、`--no-color`、`-h`、`-V`。完整參數見 [doc/CLI.md](doc/CLI.md)。

## 輸出格式與退出碼

| 格式 | 適用 | 樣貌 |
| :--- | :--- | :--- |
| `human` | 終端閱讀 | 上色訊息、統計摘要 |
| `shasum` | 相容 `sha256sum -c` | `<hex>  <path>` |
| `bsd` | BSD 工具鏈 | `<algo> (<path>) = <hex>` |
| `json` | 程式解析 | 完整報告物件／每行一個物件 |
| `csv` | 試算表 | 表頭 + 逗號分隔 |
| `ndjson` | 串流處理 | 每行一筆，尾端含摘要 |

| 退出碼 | 意義 |
| :---: | :--- |
| `0` | 成功 |
| `1` | 驗證失敗／有差異 |
| `2` | 執行或 I/O 錯誤 |
| `3` | 用法或輸入錯誤 |
| `4` | 無可處理項目 |

## 效能設計

- **單次 I/O、多種雜湊**：`-a sha256 -a blake3` 只掃描檔案一次。
- **檔案級並行**：以 `rayon` 平行處理多檔案，`-t` 可限制執行緒數。
- **有界記憶體**：讀取緩衝區夾在 4 KiB～64 MiB；BLAKE3 對大檔的整檔載入
  預設上限 512 MiB，可用 `ONEE_BLAKE3_BULK_MAX_MB` 調整。
- **串流為預設**：除 BLAKE3 在特定大小區間會走 `update_bulk` 外，其餘一律串流計算。

調校建議與量測方法見 [doc/OPTIMIZATION.md](doc/OPTIMIZATION.md)。

## 作為 Rust Library 使用

```rust
use onee_checker::algorithm::HashType;
use onee_checker::fs::list_files;
use onee_checker::hash::compute_hashes_parallel;

let files = list_files("./dist")?;
let results = compute_hashes_parallel(&files, &HashType::SHA256, 1024 * 1024);
for item in results.into_iter().flatten() {
    println!("{}  {}", item.hash_hex(), item.path.display());
}
# Ok::<(), onee_checker::error::OneeError>(())
```

模組：`algorithm`、`hash`、`hasher`、`fs`、`tree`、`report`、`manifest`、`error`。
完整 API 見 [doc/API.md](doc/API.md)。

## 支援的演算法

MD5、SHA-1、SHA-224、SHA-256、SHA-384、SHA-512、SHA3-256、SHA3-512、
SHAKE-128、SHAKE-256、BLAKE2s-256、BLAKE2b-512、BLAKE3。
可變長度演算法支援 `--length`。

## 文件索引

| 文件 | 內容 |
| :--- | :--- |
| [doc/TUTORIAL.md](doc/TUTORIAL.md) | 循序教學：從第一份清單到 CI 整合 |
| [doc/CLI.md](doc/CLI.md) | 完整指令參考、輸出格式、退出碼 |
| [doc/API.md](doc/API.md) | Library API 與型別 |
| [doc/ARCHITECTURE.md](doc/ARCHITECTURE.md) | 模組分層、資料流、設計決策 |
| [doc/OPTIMIZATION.md](doc/OPTIMIZATION.md) | 效能調校與量測 |
| [doc/SECURITY.md](doc/SECURITY.md) | 安全模型與威脅考量 |
| [doc/DEVELOPMENT.md](doc/DEVELOPMENT.md) | 開發、測試、CI、發佈流程 |
| [CHANGELOG.md](CHANGELOG.md) | 版本變更記錄 |

## 授權

MIT，見 [LICENSE](LICENSE)。版權所有 (c) 2026 逍遙 (XiaoYao)。
