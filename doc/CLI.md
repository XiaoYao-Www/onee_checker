# Onee Checker 指令參考手冊

**版本**：2.1.1 ｜ **執行檔名稱**：`oneechk`

本文件是 `oneechk` 的完整指令參考。想循序學習，請看 [TUTORIAL.md](TUTORIAL.md)；
想了解輸出欄位與程式設計介面，請看 [API.md](API.md)。

---

## 目錄

1. [呼叫慣例](#呼叫慣例)
2. [全域選項](#全域選項)
3. [子命令](#子命令)
   - [`hash`](#hash)
   - [`verify`](#verify)
   - [`check`](#check)
   - [`diff`](#diff)
   - [`json`](#json)
   - [`txt`](#txt)
4. [輸出格式詳解](#輸出格式詳解)
5. [退出碼與 CI 用法](#退出碼與-ci-用法)
6. [與 GNU coreutils 的相容性](#與-gnu-coreutils-的相容性)
7. [常見錯誤與排解](#常見錯誤與排解)

---

## 呼叫慣例

```text
oneechk [GLOBAL_OPTIONS] <COMMAND> [OPTIONS] [ARGS]
```

**串流分工**（設計契約，腳本可依賴）：

| 串流 | 內容 | 範例 |
| :--- | :--- | :--- |
| stdout | 機器可解析的資料 | 雜湊清單、JSON 報告、樹狀文字 |
| stderr | 人類可讀的訊息 | 進度條、`✔ 已儲存`、錯誤、統計摘要 |

因此 `oneechk hash . -o - | sha256sum -c -` 這類管線可以直接使用，狀態訊息不會污染資料。

## 全域選項

| 選項 | 預設 | 說明 |
| :--- | :--- | :--- |
| `-f`, `--format <FORMAT>` | `human` | 輸出格式：`human`、`shasum`、`bsd`、`json`、`csv`、`ndjson`。可放在子命令之前或之後。 |
| `--no-color` | 關閉 | 停用 ANSI 色彩（適合寫入日誌檔） |
| `-h`, `--help` | — | 顯示說明 |
| `-V`, `--version` | — | 顯示版本 |

> `human` 與 `shasum` 在 `hash` 子命令下輸出相同的 `<hex>  <path>` 格式；
> 其他子命令的 `human` 走「人類可讀」路徑，資料請改用 `json` 或 `csv`。

---

## `hash`

計算檔案或整個目錄的雜湊，並輸出/儲存清單。

```bash
oneechk hash <PATH> [OPTIONS]
```

| 選項 | 預設 | 說明 |
| :--- | :--- | :--- |
| `<PATH>` | 必填 | 檔案或目錄（目錄會遞迴列出所有檔案） |
| `-a`, `--algo <ALGO>` | `sha256` | 演算法；**可重複**（`-a sha256 -a blake3`），單次 I/O 同時計算多種雜湊 |
| `-o`, `--output <PATH>` | `<名稱>.<演算法>` | 輸出檔；`-` 表示寫到 stdout |
| `--length <N>` | 演算法預設 | 輸出長度（bytes），用於 SHAKE、BLAKE2、BLAKE3 等可變長度演算法 |
| `--buffer <SIZE>` | `1M` | 讀取緩衝區，支援 `K`/`M`/`G` 後綴（會夾在 4 KiB～64 MiB） |
| `-t`, `--threads <N>` | CPU 數 | 檔案級並行數 |
| `--digest-only` | 關閉 | stdout 只印雜湊值（多演算法時每行 `<演算法>: <hex>`） |
| `--strict` | 關閉 | 遇錯誤立即中止（預設繼續並彙總） |
| `-b`, `--binary` | 關閉 | 二進位模式：`<hex> *<path>`（對應 GNU `sha256sum -b`） |
| `--text` | 預設 | 文字模式：`<hex>  <path>`（兩個空白）；與 `--binary` 互斥 |
| `-q`, `--quiet` | 關閉 | 不顯示進度與訊息 |

> `--binary`／`--text` 同時影響 stdout 與寫入檔案的清單。
> 預設是**文字模式**（與 GNU `sha256sum` 一致）；`verify` 兩種模式都能讀。

**範例**

```bash
oneechk hash ./release                     # 產生 release.sha256
oneechk hash ./release -a sha256 -a blake3 # 產生 release.blake3（兩種雜湊各一輪）
oneechk hash ./release -o -                 # 清單直接寫到 stdout
oneechk hash ./asset.bin --digest-only      # 只印出雜湊值
```

**退出碼**：`0` 成功；`2` 有檔案無法讀取；`3` 參數錯誤；`4` 目錄中沒有檔案。

---

## `verify`

依清單驗證檔案完整性。

```bash
oneechk verify <HASHFILE> [OPTIONS]
```

| 選項 | 預設 | 說明 |
| :--- | :--- | :--- |
| `<HASHFILE>` | 必填 | 清單檔（shasum 格式），副檔名用於推斷演算法 |
| `-a`, `--algo <ALGO>` | 由副檔名推斷 | 明確指定演算法（例如清單命名為 `checksums.txt` 時） |
| `-r`, `--root <DIR>` | 清單所在目錄 | 檔案搜尋的根目錄 |
| `--ignore-missing` | 關閉 | 檔案不存在時記為 `missing`，不算錯誤 |
| `--report <PATH>` | — | 額外輸出完整 JSON 報告（不論 `--format` 為何） |
| `--buffer <SIZE>` | `1M` | 讀取緩衝區 |
| `-t`, `--threads <N>` | CPU 數 | 檔案級並行數 |
| `--strict` | 關閉 | 遇錯誤立即中止 |
| `-b`, `--binary` | 關閉 | 以二進位模式重新輸出清單（配合 `--format shasum`／`bsd`） |
| `--text` | 預設 | 以文字模式重新輸出清單；與 `--binary` 互斥 |
| `-q`, `--quiet` | 關閉 | 只輸出最終統計 |

**語意**：`verify` 會先嚴格解析清單；清單格式錯誤或沒有任何有效項目時直接回 `3`，
不會回報「全部通過」。

**範例**

```bash
oneechk verify release.sha256
oneechk verify release.sha256 -r ./dist
oneechk verify release.sha256 --format json > report.json
oneechk verify release.sha256 --report report.json
```

**退出碼**：`0` 全部相符；`1` 有不相符；`2` 讀取檔案或清單時發生 I/O 錯誤；`3` 清單格式錯誤；
`4` 沒有可處理項目。

---

## `check`

驗證單一檔案或一段文字的雜湊，不需要清單檔。

```bash
oneechk check <PATH> --expect <HEX>... [OPTIONS]
oneechk check --text <STRING> --expect <HEX>... [OPTIONS]
```

| 選項 | 預設 | 說明 |
| :--- | :--- | :--- |
| `<PATH>` | 與 `--text` 互斥 | 要驗證的檔案 |
| `--text <STRING>` | 與 `PATH` 互斥 | 要驗證的文字內容（UTF-8 位元組） |
| `-e`, `--expect <HEX>` | 必填，可重複 | 期望的雜湊值；**任一相符即通過** |
| `-a`, `--algo <ALGO>` | `sha256` | 使用的演算法 |
| `--length <N>` | 演算法預設 | 輸出長度（bytes） |
| `--ignore-case` | 關閉 | 比較時忽略十六進位大小寫 |

**範例**

```bash
oneechk check ./download.zip -e 8ed3f6ad...
oneechk check --text "hello" -a sha256 \
  -e 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
oneechk check ./download.zip -e 8ED3F6AD... --ignore-case
```

**退出碼**：`0` 相符；`1` 不相符；`2` I/O 錯誤；`3` 參數或路徑錯誤。

---

## `diff`

比對兩份清單，找出新增、移除與變更的檔案。

```bash
oneechk diff <OLD> <NEW> [--format human|csv|json]
```

| 參數 | 說明 |
| :--- | :--- |
| `<OLD>` | 舊清單 |
| `<NEW>` | 新清單 |

清單格式會自動偵測（shasum／BSD／CSV／JSON），以相對路徑為鍵、十六進位比較不分大小寫。

**human 輸出**（stdout）：

```text
+ data/c.txt
- old/removed.txt
~ data/a.txt 8ed3f6ad... -> 1b2a9c11...
```

摘要寫到 stderr：`差異: 1 新增, 0 移除, 0 變更, 2 未變`。

**退出碼**：`0` 兩份完全相同；`1` 有差異；`2` I/O 錯誤；`3` 清單無法解析。

---

## `json`

輸出目錄結構的 JSON。

```bash
oneechk json <PATH> [-o <OUTPUT>] [-q]
```

| 選項 | 預設 | 說明 |
| :--- | :--- | :--- |
| `<PATH>` | 必填 | 目錄 |
| `-o`, `--output <PATH>` | `<名稱>.struct.json` | `-` 表示 stdout |
| `-q`, `--quiet` | 關閉 | 不顯示訊息 |

輸出結構：

```json
{
  "version": "2.1.1",
  "generation_time": 1790000000,
  "nodes": [
    { "name": "data", "path": "data", "is_dir": true, "children": [] }
  ]
}
```

## `txt`

輸出樹狀目錄文字（適合終端閱讀或貼進 issue）。

```bash
oneechk txt <PATH> [-o <OUTPUT>] [-s binary|decimal|raw] [-m] [-c] [-q]
```

| 選項 | 預設 | 說明 |
| :--- | :--- | :--- |
| `-o`, `--output <PATH>` | `<名稱>.tree.txt` | `-` 表示寫到 stdout（不加時寫入檔案，不印到終端機） |
| `-s`, `--size <MODE>` | 不顯示 | `binary`（KiB）、`decimal`（KB）、`raw`（bytes） |
| `-m`, `--modified` | 關閉 | 顯示最後修改時間 |
| `-c`, `--created` | 關閉 | 顯示建立時間 |

輸出開頭會有三行註解（分隔線與產生時間），主體為樹狀結構：

```text
# ******************************
# Generation Time: 2026-10-24 13:56:41 +0800
# ******************************

data (9 B)
├── a.txt (5 B)
└── b.txt (4 B)
```

---

## 輸出格式詳解

以下為實際輸出（`data/` 內含 `a.txt`、`b.txt`）。

### `hash` 的 `shasum`／`human`

預設是**文字模式**（兩個空白）：

```text
8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8  data/a.txt
f44e64e75f3948e9f73f8dfa94721c4ce8cbb4f265c4790c702b2d41cfbf2753  data/b.txt
```

加上 `-b`／`--binary` 則為**二進位模式**（星號）：

```text
8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8 *data/a.txt
f44e64e75f3948e9f73f8dfa94721c4ce8cbb4f265c4790c702b2d41cfbf2753 *data/b.txt
```

- 路徑一律以 `/` 分隔（跨平台一致）。
- 兩種模式都能直接被 `sha256sum -c` 驗證；`oneechk verify` 也兩種都接受。
- 模式只影響 shasum 系列；`bsd` 格式沒有模式欄位，`csv` 也不變。

### `verify` 的 `shasum`／`bsd`（重新輸出清單）

`verify --format shasum` 會把每筆「實際 hash」重新輸出成清單，可用來正規化既有清單：

```bash
oneechk verify data.sha256 -q --format shasum -b
```

```text
8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8 *data/a.txt
f44e64e75f3948e9f73f8dfa94721c4ce8cbb4f265c4790c702b2d41cfbf2753 *data/b.txt
```

### `hash` 的 `bsd`

```text
sha-256 (data/a.txt) = 8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8
```

### `hash` 的 `csv`

```csv
8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8,data/a.txt
```

### `hash` 的 `json` / `ndjson`

每行一個物件（NDJSON 風格，適合串流處理）；`mode` 會反映目前模式：

```json
{"algorithm":"sha-256","hash":"8ed3f6ad...","mode":"text","path":"data/a.txt"}
```

### `verify` 的 `json`（完整報告）

```json
{
  "version": "2.1.1",
  "command": "verify",
  "hashfile": "data.sha256",
  "algorithm": "SHA-256",
  "root": "C:\\work\\demo",
  "summary": { "total": 2, "matched": 2, "mismatched": 0, "errors": 0, "missing": 0 },
  "exit_code": 0,
  "entries": [
    {
      "path": "data/a.txt",
      "status": "match",
      "expected": "8ed3f6ad...",
      "actual": "8ed3f6ad...",
      "error": null
    }
  ]
}
```

`entries[].status` 可能值：`match`、`mismatch`、`error`、`missing`。

### `verify` 的 `ndjson`

逐筆 entry 一行，最後一行為摘要：

```json
{"path":"data/a.txt","status":"match","expected":"8ed3f6ad...","actual":"8ed3f6ad...","error":null}
{"summary":{"total":2,"matched":2,"mismatched":0,"errors":0,"missing":0}}
```

### `verify` 的 `csv`

```csv
path,status,expected,actual
data/a.txt,match,8ed3f6ad...,8ed3f6ad...
```

### `check` 的 `json`

```json
{"path":"data/a.txt","status":"match","expected":"8ed3f6ad...","actual":"8ed3f6ad...","error":null}
```

### `diff` 的 `json`

```json
{"added":["data/c.txt"],"removed":[],"changed":[],"unchanged":2}
```

`changed` 的每一項為 `{"path","old_hash","new_hash"}`。

### `diff` 的 `csv`

```csv
kind,path,old_hash,new_hash
added,data/c.txt,,
changed,data/a.txt,8ed3f6ad...,1b2a9c11...
```

---

## 退出碼與 CI 用法

| 代碼 | 意義 | 常見來源 |
| :---: | :--- | :--- |
| `0` | 成功 | 全部相符／無差異 |
| `1` | 驗證失敗 | `verify` 有不符、`check` 不匹配、`diff` 有差異 |
| `2` | 執行或 I/O 錯誤 | 檔案讀取失敗、權限不足 |
| `3` | 用法或輸入錯誤 | 參數錯誤、清單格式錯誤、路徑不存在 |
| `4` | 無可處理項目 | 目錄內沒有檔案 |

**GitHub Actions / bash**

```bash
set -euo pipefail
cargo install --path .
oneechk hash ./dist -a sha256
oneechk verify dist.sha256 -r ./dist --format json > verify.json
```

**PowerShell**

```powershell
$ErrorActionPreference = 'Stop'
oneechk verify dist.sha256 -r ./dist -q
if ($LASTEXITCODE -ne 0) { throw "驗證失敗（exit=$LASTEXITCODE）" }
```

> 只有 `0` 代表完全成功。若只需要「有差異就讓 CI 失敗」，直接依賴退出碼即可，
> 不要解析 stderr 的文字。

## 與 GNU coreutils 的相容性

- `hash` 的 `shasum` 輸出 `<hex>  <path>` 與 `sha256sum` 相同，可用來產生可被 `sha256sum -c` 驗證的清單。
- `hash -b` 對應 GNU `sha256sum -b`（`<hex> *<path>`）；兩者產生的清單可互換。
- `verify` 接受 `sha256sum` 產生的清單，`*`（二進位模式）與兩個空白（文字模式）前綴都會被正確處理。
- 差異：`oneechk` 支援多演算法與可變長度、輸出多種格式、退出碼分成四類；
  `sha256sum -c` 只有成功/失敗二值。

## 常見錯誤與排解

| 訊息 | 原因 | 處置 |
| :--- | :--- | :--- |
| `無效路徑: 路徑不存在或不是目錄` | `hash`/`json`/`txt` 的目標不存在 | 確認路徑；`verify`/`check` 請用檔案路徑 |
| `無法從副檔名推斷演算法，請使用 --algo 手動指定` | 清單副檔名不是 `sha256`、`md5` 等 | 加 `-a sha256` |
| `manifest 沒有有效項目` | 清單為空或格式不符 | 確認清單內容，或改用 `--format shasum` 產生 |
| `無可處理的項目`（exit 4） | 目錄內沒有檔案 | 確認目錄；空目錄不是錯誤，但沒有可計算項 |
| 驗證結果出現 `expected: null` | 清單中的相對路徑與實際檔案路徑不一致 | 用 `-r` 指定正確根目錄 |
| 路徑含逗號的清單被誤判格式 | 自動偵測以逗號判斷 CSV | 改用明確格式的清單，或先以 `--format shasum` 重新產生 |
