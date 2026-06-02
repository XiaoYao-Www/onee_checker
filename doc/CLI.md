# Onee Checker 指令參考手冊 (CLI)

**版本**: 2.0.0  
**二進位名稱**: `oneechk`

---

## 概覽

```
oneechk <COMMAND> [OPTIONS]

Commands:
  hash    生成 hash 驗證檔
  verify  驗證 hash 驗證檔
  json    生成 JSON 目錄結構紀錄
  txt     生成文字樹狀目錄結構
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

---

## 1. `oneechk hash` — 生成 hash 驗證檔

計算指定路徑下所有檔案的密碼學雜湊值，輸出 `shasum` 相容格式。

### 語法

```
oneechk hash [OPTIONS] <PATH>
```

### 參數

| 參數 | 短 | 長 | 類型 | 預設 | 說明 |
|---|---|---|---|---|---|
| `<PATH>` | | | `PathBuf` | (必需) | 要計算的檔案或目錄路徑 |
| `--algo` | `-a` | `--algo` | `HashAlgo[]` | `sha256` | 演算法，可多次指定 |
| `--output` | `-o` | `--output` | `PathBuf` | auto | 輸出檔案。指定 `-` 輸出到 stdout |
| `--length` | | `--length` | `u16` | 預設值 | 可變長度演算法的輸出長度 (bytes) |
| `--buffer` | | `--buffer` | `BufferSize` | `1M` | 讀取緩衝區大小 (支援 K/M/G 後綴) |
| `--threads` | `-t` | `--threads` | `usize` | CPU cores | 並行線程數 |
| `--quiet` | `-q` | `--quiet` | flag | false | 不顯示進度條 |

### 支援的演算法

| 值 | 演算法 | 輸出長度 (bits) | 副檔名 | 可變長度 |
|---|---|---|---|---|
| `md5` | MD5 | 128 | `.md5` | ❌ |
| `sha1` | SHA-1 | 160 | `.sha1` | ❌ |
| `sha224` | SHA-224 | 224 | `.sha224` | ❌ |
| `sha256` | SHA-256 | 256 | `.sha256` | ❌ |
| `sha384` | SHA-384 | 384 | `.sha384` | ❌ |
| `sha512` | SHA-512 | 512 | `.sha512` | ❌ |
| `sha3-256` | SHA3-256 | 256 | `.sha3_256` | ❌ |
| `sha3-512` | SHA3-512 | 512 | `.sha3_512` | ❌ |
| `shake128` | SHAKE-128 | 256 | `.shake128-<bits>` | ✅ |
| `shake256` | SHAKE-256 | 512 | `.shake256-<bits>` | ✅ |
| `blake2s256` | BLAKE2s-256 | 256 | `.blake2s256` | ❌ |
| `blake2b512` | BLAKE2b-512 | 512 | `.blake2b512` | ❌ |
| `blake3` | BLAKE3 | 256 | `.blake3-<bits>` | ✅ |

### 輸出格式

```
# ******************************
# Total Files Count: 878
# Generation Time: 2026-06-02 15:30:00 +0800
# ******************************
<hash_hex> *<relative_path>
```

- 註解行以 `#` 開頭
- 每筆記錄格式: `<hex_hash> *<relative_path>`
- `*` 表示二進位模式 (binary mode)，相容於 GNU/BusyBox `sha*sum`

### 範例

```bash
# 預設 SHA-256 計算
oneechk hash ./Downloads

# 輸出: ./Downloads.sha256

# 多重算法
oneechk hash ./Downloads -a sha256 -a md5 -a blake3

# 輸出: ./Downloads.sha256, ./Downloads.md5, ./Downloads.blake3-256

# stdout 輸出
oneechk hash ./Downloads -o - -q > checksums.txt

# BLAKE3 自訂 64 bytes 輸出
oneechk hash ./Downloads -a blake3 --length 64

# 輸出: ./Downloads.blake3-512 (64 bytes × 8 = 512 bits)

# 效能調校
oneechk hash ./large-dir --buffer 8K -t 4
```

### 錯誤案例

| 情境 | Exit Code | 訊息 |
|---|---|---|
| 路徑不存在 | 3 | `無效路徑: 路徑不存在: <path>` |
| 固定長度算法 + `--length` | 3 | `算法不支援自訂輸出長度: Sha256` |
| 空目錄 | 0 | `⚠ 警告: 指定路徑下無任何檔案` |
| 無法寫入輸出 | 2 | `I/O 錯誤: ...` |

---

## 2. `oneechk verify` — 驗證 hash 檔

解析 hash 驗證檔，逐一比對檔案完整性。演算法預設從副檔名自動推斷。

### 語法

```
oneechk verify [OPTIONS] <HASHFILE>
```

### 參數

| 參數 | 短 | 長 | 類型 | 預設 | 說明 |
|---|---|---|---|---|---|
| `<HASHFILE>` | | | `PathBuf` | (必需) | hash 驗證檔路徑 |
| `--algo` | `-a` | `--algo` | `HashAlgo` | auto | 手動指定演算法 |
| `--root` | `-r` | `--root` | `PathBuf` | hash 檔目錄 | 相對路徑的基準目錄 |
| `--buffer` | | `--buffer` | `BufferSize` | `1M` | 讀取緩衝區大小 |
| `--threads` | `-t` | `--threads` | `usize` | CPU cores | 並行線程數 |
| `--quiet` | `-q` | `--quiet` | flag | false | 僅顯示最終摘要 |

### 演算法推斷規則

當未指定 `--algo` 時，從 hash 檔副檔名推斷：

| 副檔名 | 演算法 |
|---|---|
| `.md5` | MD5 |
| `.sha1` | SHA-1 |
| `.sha256` | SHA-256 |
| `.sha512` | SHA-512 |
| `.sha3_256` | SHA3-256 |
| `.blake3-*` | BLAKE3 |
| ... | ... |

### 輸出

```
ℹ 驗證 checksums.sha256 （演算法: SHA-256 根目錄: /data）

✔ /data/file1.txt
✔ /data/file2.bin
✘ /data/file3.log  預期 hash 不匹配 (實際: abcd1234...)

✔ 驗證完成: 878 個檔案, 877 匹配, 1 不匹配, 0 錯誤
```

### 安全防護

- **Path Traversal**: `../../etc/passwd` 型路徑會立即被拒絕
- **絕對路徑**: `/etc/shadow` 型路徑會被拒絕
- **非法 hex**: 非十六進位 hash 行被跳過並警告

### 退出碼

| Code | 含義 |
|---|---|
| 0 | 所有檔案 hash 匹配 |
| 1 | 至少一個檔案 hash 不匹配 |
| 2 | I/O 錯誤 |
| 3 | 參數錯誤 / hash 檔格式錯誤 |

---

## 3. `oneechk json` — JSON 目錄結構

遞迴掃描目錄，輸出結構化 JSON 紀錄。

### 語法

```
oneechk json [OPTIONS] <PATH>
```

### JSON 結構

```json
{
  "version": "0.1.0",
  "generation_time": 1717200000,
  "nodes": [{
    "name": "dirname",
    "is_dir": true,
    "is_symlink": false,
    "extension": null,
    "size": 1048576,
    "last_modified": 1717200000,
    "created_at": 1717200000,
    "children": [...],
    "symlink_target": null
  }]
}
```

### 欄位說明

| 欄位 | 類型 | 說明 |
|---|---|---|
| `name` | String | 節點名稱 |
| `is_dir` | Bool | 是否為目錄 |
| `is_symlink` | Bool | 是否為符號連結 |
| `extension` | String or null | 副檔名 (小寫) |
| `size` | u64 | 大小 (bytes)。目錄為所有子節點總和 |
| `last_modified` | i64 or null | 最後修改時間 (Unix timestamp) |
| `created_at` | i64 or null | 建立時間 (Unix timestamp) |
| `children` | Array or null | 子節點列表 (目錄才有) |
| `symlink_target` | String or null | 符號連結目標 |

---

## 4. `oneechk txt` — 樹狀目錄結構

輸出人類可讀的樹狀目錄結構。

### 語法

```
oneechk txt [OPTIONS] <PATH>
```

### 大小顯示格式

| 值 | 說明 | 範例 |
|---|---|---|
| `binary` | 1024 為底 | `1.00 KiB`, `2.50 MiB` |
| `decimal` | 1000 為底 | `1.00 KB`, `2.50 MB` |
| `raw` | 原始位元組 + 逗號 | `1,234 B`, `1,234,567 B` |

### 輸出編碼

- 樹枝符號使用 Unicode `├──`, `└──`, `│`
- 時間格式: `YYYY-MM-DD HH:MM:SS +TZ`
- 空欄位不顯示

---

## 5. 共用特性

### Buffer Size 格式

`--buffer` 參數支援人類可讀的後綴：

| 後綴 | 倍數 | 範例 |
|---|---|---|
| (無) | 1 | `4096` = 4096 bytes |
| `K` | 1024 | `4K` = 4096 bytes |
| `M` | 1024² | `1M` = 1,048,576 bytes |
| `G` | 1024³ | `2G` = 2,147,483,648 bytes |

### 安靜模式 (`-q`)

- 不顯示進度條 (`hash`)
- 不顯示逐檔訊息 (`verify`, 僅顯示最終摘要)
- 不顯示儲存成功訊息 (`json`, `txt`)

### stdout 輸出 (`-o -`)

指定 `-o -` 時，資料輸出到 stdout 而非檔案：

```bash
oneechk hash /data -o - -q | sha256sum -c
oneechk json /data -o - -q | jq '.nodes[0].children | length'
```

---

## 6. 進階用法

### CI/CD 管線中的完整性驗證

```bash
#!/bin/bash
# 生成 hash
oneechk hash ./dist -a sha256 -o dist.sha256 -q

# 傳輸後驗證
oneechk verify dist.sha256 -r ./received -q
if [ $? -ne 0 ]; then
    echo "❌ 檔案完整性驗證失敗！"
    exit 1
fi
echo "✅ 驗證通過"
```

### 目錄快照比對

```bash
# 生成快照
oneechk json /data -o snapshot_before.json -q

# ... 目錄變更 ...

# 生成新快照並比對
oneechk json /data -o snapshot_after.json -q
diff <(jq -S . snapshot_before.json) <(jq -S . snapshot_after.json)
```

### 大量檔案最佳化

```bash
# SSD 環境：較大 buffer 減少 syscall
oneechk hash /ssd/data --buffer 4M -t 8

# HDD 環境：減少線程避免磁頭 thrashing
oneechk hash /hdd/data --buffer 64K -t 1
```
