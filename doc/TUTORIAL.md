# 教學：用 oneechk 建立可重複的檔案完整性流程

本教學從零開始，帶你完成「產生清單 → 驗證 → 放進 CI → 嵌入程式」的完整流程。
所有指令與輸出都在 Windows（PowerShell）與 Linux（bash）上實際執行過；若你的輸出略有差異
（例如雜湊值），屬於正常現象。

**前置需求**：已安裝 `oneechk`（見 [README](../README.md#安裝)），終端機可執行 `oneechk --version`。

---

## 第 0 章：確認環境

```bash
oneechk --version
```

```text
oneechk 2.1.0
```

準備一個練習目錄：

```powershell
# PowerShell
New-Item -ItemType Directory -Force demo\dist | Out-Null
Set-Content demo\dist\a.txt -Value 'alpha' -NoNewline
Set-Content demo\dist\b.txt -Value 'beta'  -NoNewline
```

```bash
# bash
mkdir -p demo/dist
printf 'alpha' > demo/dist/a.txt
printf 'beta'  > demo/dist/b.txt
```

本篇會用到三個名詞：

| 名詞 | 意思 |
| :--- | :--- |
| 清單（manifest） | 每行 `<雜湊>  <相對路徑>` 的文字檔 |
| 根目錄（root） | 清單中相對路徑的基準；預設是清單所在目錄 |
| 退出碼 | `0` 成功、`1` 驗證失敗、`2` I/O 錯誤、`3` 用法錯誤、`4` 無可處理項目 |

## 第 1 章：產生第一份清單

```bash
cd demo
oneechk hash ./dist
```

```text
✔ 已儲存: .\dist.sha256
```

看一下內容：

```bash
cat dist.sha256
```

```text
8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8  dist/a.txt
f44e64e75f3948e9f73f8dfa94721c4ce8cbb4f265c4790c702b2d41cfbf2753  dist/b.txt
```

重點：

- 雜湊與路徑之間是**兩個空白**，與 `sha256sum` 相同格式。
- 路徑一律使用 `/`，即使你在 Windows 上執行，清單仍可跨平台驗證。
- 清單檔名預設是 `<目錄名>.<演算法>`；用 `-o -` 可改寫到 stdout。

```bash
oneechk hash ./dist -o - | head -n 1
```

### 文字模式與二進位模式

GNU `sha256sum` 用分隔符區分模式，`oneechk` 也支援：

```bash
oneechk hash ./dist/a.txt -a sha256 -o - -q       # 預設：文字模式
# 8ed3f6ad...  a.txt

oneechk hash ./dist/a.txt -a sha256 -o - -q -b    # 二進位模式
# 8ed3f6ad... *a.txt
```

- `-b`／`--binary`：二進位模式（`<hex> *<path>`），對應 `sha256sum -b`。
- `--text`：明確指定文字模式（預設）；與 `-b` 同時使用會回 exit 3。
- 兩種清單 `verify` 都吃；`--format json` 會多一個 `mode` 欄位可判斷來源。
- 預設模式是**文字模式**，與 GNU coreutils 一致。

## 第 2 章：驗證與讀懂退出碼

```bash
oneechk verify dist.sha256
```

```text
✔ 驗證完成: 2 個檔案, 2 匹配, 0 不匹配, 0 錯誤
```

現在改一個位元組，再驗證一次：

```bash
printf 'alphA' > dist/a.txt
oneechk verify dist.sha256 -q; echo "exit=$?"
```

```text
✘ 驗證完成: 2 個檔案, 1 匹配, 1 不匹配, 0 錯誤
exit=1
```

把檔案改回來，驗證就恢復：

```bash
printf 'alpha' > dist/a.txt
oneechk verify dist.sha256 -q; echo "exit=$?"
# 0
```

**為什麼退出碼比文字可靠？** `-q` 只留下最終統計，而退出碼在任何語系、任何版本都不變。
腳本請一律判斷退出碼，不要解析訊息。

### 缺檔與嚴格模式

```bash
rm dist/b.txt
oneechk verify dist.sha256 -q; echo "exit=$?"          # 2（讀取失敗）
oneechk verify dist.sha256 -q --ignore-missing; echo $? # 0（缺檔不算錯）
```

## 第 3 章：多演算法與單次 I/O

同時要 SHA-256 與 BLAKE3：

```bash
oneechk hash ./dist/a.txt -a sha256 -a blake3 -o - -q
```

```text
8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8  a.txt
644a9bc57c6063e2ba4028fa73ed585170ae7db8ac7723d32be49c021a0225f5  a.txt
```

兩輪輸出的順序與 `-a` 的順序一致。要注意 `shasum` 格式本身沒有演算法欄位，
所以多演算法時兩行「長得一樣」；要能分辨請改用 `--digest-only` 或 `--format json`。

關鍵是這**只掃描檔案一次**：`oneechk` 在同一次讀取中把多個 hasher 一起 update，
檔案越大、演算法越多，省下的 I/O 越多。

只要雜湊值、不要清單格式：

```bash
oneechk hash ./dist/a.txt --digest-only
```

```text
8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8
```

多演算法時每行會加演算法前綴：

```bash
oneechk hash ./dist/a.txt -a sha256 -a blake3 --digest-only
```

```text
SHA-256: 8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8
BLAKE3: 644a9bc57c6063e2ba4028fa73ed585170ae7db8ac7723d32be49c021a0225f5
```

## 第 4 章：目錄盤點

### 樹狀文字（給人看）

```bash
oneechk txt ./dist -s binary -o -
```

```text
# ******************************
# Generation Time: 2026-10-24 13:56:41 +0800
# ******************************

dist (9 B)
├── a.txt (5 B)
└── b.txt (4 B)
```

- 不加 `-o -` 時會寫入 `<目錄名>.tree.txt`，不會印到終端機。
- 加上 `-m` 會附上最後修改時間：`a.txt (5 B) [mod: 2026-10-24 13:50:39 +08:00]`。
- `-s` 可選 `binary`（KiB）、`decimal`（KB）、`raw`（bytes）。

### JSON（給機器看）

```bash
oneechk json ./dist -o -
```

```json
{
  "version": "2.1.0",
  "generation_time": 1792821395,
  "nodes": [
    {
      "name": "dist",
      "is_dir": true,
      "is_symlink": false,
      "extension": null,
      "size": 9,
      "last_modified": 1792821039,
      "created_at": 1792821013,
      "children": [
        { "name": "a.txt", "is_dir": false, "size": 5, "extension": "txt" }
      ]
    }
  ]
}
```

`generation_time`、`last_modified`、`created_at` 都是 Unix epoch 秒數，
時間欄位不存在時為 `null`。

## 第 5 章：放進 CI 與腳本

### bash（GitHub Actions / Linux）

```bash
set -euo pipefail

oneechk hash ./artifacts -a sha256 -o artifacts.sha256 -q
oneechk verify artifacts.sha256 -r ./artifacts -q --format json > verify.json
```

`verify` 在不相符時退出碼為 `1`，`set -e` 會讓 job 直接失敗；`verify.json` 可上傳為 artifact。

### PowerShell

```powershell
$ErrorActionPreference = 'Stop'
oneechk verify artifacts.sha256 -r ./artifacts -q --format json | Set-Content verify.json
if ($LASTEXITCODE -ne 0) { throw "驗證失敗（exit=$LASTEXITCODE）" }
```

### 只取需要的欄位

```bash
oneechk verify artifacts.sha256 -q --format ndjson |
  jq -r 'select(.summary) | "\(.summary.mismatched) 筆不符"'
```

### 版本之間比對

```bash
oneechk diff v1.sha256 v2.sha256
```

```text
+ dist/c.txt
~ dist/a.txt 8ed3f6ad... -> 1b2a9c11...
差異: 1 新增, 0 移除, 1 變更, 1 未變
```

（`+`/`-`/`~` 走 stdout，摘要走 stderr。）

## 第 6 章：作為 Library 嵌入

```toml
[dependencies]
onee_checker = { path = "../onee_checker" }
```

```rust
use onee_checker::algorithm::HashType;
use onee_checker::fs::list_files;
use onee_checker::hash::compute_hashes_parallel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = list_files("./dist")?;
    let results = compute_hashes_parallel(&files, &HashType::SHA256, 1024 * 1024);

    for item in results.into_iter().flatten() {
        println!("{}  {}", item.hash_hex(), item.path.display());
    }
    Ok(())
}
```

想直接產生 JSON 報告，`report` 模組提供 `VerifyReport` 與 `write_verify_report`；
清單解析與比對則在 `manifest`（`parse_auto`、`diff`）。詳見 [API.md](API.md)。

## 第 7 章：疑難排解

| 症狀 | 原因 | 處理 |
| :--- | :--- | :--- |
| `exit=2` 且訊息提到讀取失敗 | 檔案被刪除或權限不足 | 確認檔案存在；必要時用 `--ignore-missing` |
| `exit=3`、`manifest 沒有有效項目` | 清單格式不符或為空 | 用 `oneechk hash ... -o -` 重新產生 |
| `exit=4` | 目標目錄沒有檔案 | 確認路徑；空目錄本身不是錯誤 |
| 驗證結果 `expected` 為 `null` | 清單中的相對路徑與實際檔案不一致 | 用 `-r` 指定正確根目錄 |
| JSON 被其他訊息污染 | 誤把 stderr 重導進 stdout | 只重導 stdout：`oneechk ... 2>log.txt` |
| Windows 路徑出現 `\\?\` | 直接使用 `canonicalize` 的結果 | 2.1.0 起報告已去除前綴；自訂腳本請沿用報告中的 `root`/`path` |

---

**下一步**：完整的參數與輸出格式見 [CLI.md](CLI.md)；效能調校見 [OPTIMIZATION.md](OPTIMIZATION.md)。
