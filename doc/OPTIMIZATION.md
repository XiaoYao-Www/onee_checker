# 效能調校指南

本文件說明 `oneechk` 的效能模型、可調參數與量測方法。原則：**先量測、再調參**；
不要憑感覺調整 `--threads` 或 `--buffer`。

---

## 1. 效能模型

`oneechk` 的雜湊計算可拆成三層成本：

| 層次 | 成本來源 | 影響因素 | 可調參數 |
| :--- | :--- | :--- | :--- |
| 檔案系統 | `open`/`read` 系統呼叫、頁快取命中 | 檔案數量、單檔大小、儲存裝置 | `--buffer` |
| 雜湊計算 | CPU 週期 | 演算法、輸出長度、單核頻率 | `-a`、`--length` |
| 排程 | 執行緒建立與工作分配 | 檔案數 vs 核心數 | `-t`、`--threads` |

三個關鍵設計：

1. **檔案級並行（rayon）**：以 `par_iter` 把「每個檔案」分散到工作竊取佇列；
   每個 worker 只配置一次讀取緩衝區與 hasher 模板（`map_init`），之後每個檔案 `clone`。
2. **單次 I/O、多演算法**：`-a sha256 -a blake3` 只掃描檔案一次，同一塊資料依序餵給所有 hasher。
   檔案大、演算法多時，省下的 I/O 成本最明顯。
3. **BLAKE3 的特殊路徑**：小於 256 MiB 走串流；介於 256 MiB 與上限（預設 512 MiB）之間
   整檔載入並用 `update_bulk` 啟用 BLAKE3 內部多執行緒樹狀雜湊；超過上限回到串流。

## 2. 參數調校表

| 參數 | 預設 | 建議 | 說明 |
| :--- | :--- | :--- | :--- |
| `--buffer` | `1M` | 小檔多 → 64K～256K；大檔 → 4M～16M | 值越大系統呼叫越少，但每個 worker 各佔一份記憶體；下限 4 KiB、上限 64 MiB |
| `-t` / `--threads` | CPU 邏輯核心數 | 與磁碟或網路頻寬相當即可 | 超過儲存頻寬後只會增加競爭；容器環境建議設為 CPU limit |
| `-a` | `sha256` | 只算需要的 | 每多一種演算法只增加 CPU，不增加 I/O |
| `--length` | 演算法預設 | 依需求 | 只影響輸出位元組數，對速度影響有限 |
| `--digest-only` | 關閉 | 腳本取用時開啟 | 省下格式化成本（影響極小，但輸出更乾淨） |

**記憶體估算**：約 `執行緒數 × buffer 大小 + (BLAKE3 整檔載入時的檔案大小)`。
例：8 執行緒、1M buffer → 約 8 MB 固定成本；若同時處理一個 400 MiB 的 BLAKE3 檔案，
峰值會再多約 400 MiB。

## 3. 建置期最佳化

```bash
# 針對本機 CPU 指令集最佳化（不可跨機散佈）
RUSTFLAGS="-C target-cpu=native" cargo build --release

# 一般可散佈版本
cargo build --release
```

- Windows 上預設 target 由 `~/.cargo/config.toml` 決定（本專案開發機為 `x86_64-pc-windows-gnu`）；
  MSVC target 與 GNU target 的效能差異通常小於 5%，以相容性需求為準。
- `[profile.release]` 未特別調整（使用 Cargo 預設 `opt-level = 3`、無 LTO）。
  若要極致效能可自行加 `lto = "thin"`、`codegen-units = 1`，但會顯著增加編譯時間。
- 交叉編譯（musl、aarch64）請用 `cross build --release --target ...`，與 CI 一致。

## 4. 量測方法

### 4.1 端到端耗時（Windows）

```powershell
$env:TEMP="$PWD\target\_tmp"; $env:TMP=$env:TEMP
1..3 | ForEach-Object {
  Measure-Command { .\target\release\oneechk.exe hash .\TEST -a sha256 -q } |
    Select-Object -ExpandProperty TotalSeconds
}
```

### 4.2 端到端耗時（Linux / macOS）

```bash
for i in 1 2 3; do
  /usr/bin/time -f '%e s  %M KB' ./target/release/oneechk hash ./data -a sha256 -q
done
```

`%M` 是峰值 RSS（KB），可用來驗證 BLAKE3 整檔載入的記憶體上限是否生效。

### 4.3 微基準（criterion）

```bash
cargo bench --bench hash_benchmark
```

基準涵蓋單檔雜湊、多檔並行與多演算法單次 I/O。比較兩次結果時請固定：
相同的檔案集、相同的電源模式、關閉其他負載，且至少跑三輪取中位數。

## 5. 已知限制

- 讀取緩衝區與 BLAKE3 整檔上限是**全域固定**值（後者可用 `ONEE_BLAKE3_BULK_MAX_MB` 覆寫），
  不是依可用記憶體自動調整。
- 大量極小檔案（KB 級）時，`open`/`read` 的系統呼叫成本占比高，`--buffer` 的效果有限；
  此時提升併發度（`-t`）比加大 buffer 更有效。
- 多演算法只看一次 I/O，但若把輸出寫成多份清單，仍可能再次讀取檔案（例如分開執行 `hash`）。
- 磁碟為 HDD 時，並行讀取可能因 seek 而變慢；建議 `-t 1`～`-t 2`。
- 尚未提供「增量驗證」（只驗證 mtime 變更的檔案），每次 `verify` 都會完整重算。
