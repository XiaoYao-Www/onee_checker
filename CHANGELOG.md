# 變更記錄

本專案遵循 [Semantic Versioning](https://semver.org/lang/zh-TW/)。
版本號格式：`MAJOR.MINOR.PATCH`；CLI 的退出碼與輸出格式屬於公開契約。

## [未發佈]

### 新增

- `hash -b`／`--binary` 與 `hash --text`：切換清單的文字／二進位模式。
  文字模式為 `<hex>  <path>`（兩個空白），二進位模式為 `<hex> *<path>`，
  與 GNU `sha256sum`／`sha256sum -b` 一致。
- `verify -b`／`--binary`、`verify --text`：`verify --format shasum`／`bsd`
  重新輸出清單時可指定模式（先前 `verify --format shasum` 不輸出任何內容）。
- `HashMode` 型別（`onee_checker::algorithm`），`save_hash_file`、
  `report::write_hash_line`、`write_entry`、`write_verify_report` 皆新增 `mode` 參數。

### 變更

- `hash` 寫入檔案時的預設模式由二進位（` *`）改為文字（`  `），
  與 stdout 及 GNU `sha256sum` 一致；需要舊行為請加 `-b`。
- `hash --format json`／`ndjson` 每行多一個 `mode` 欄位（`text`／`binary`）。
- `--binary` 與 `--text` 互斥，同時指定會以 exit 3 結束。

### 測試

- 新增單元測試（模式分隔符、JSON `mode` 欄位、`verify --format shasum` 模式）
  與整合測試（`-b` 輸出、互斥參數、二進位清單可驗證、重新輸出模式）。

## [2.1.0] - 2026-10-24

### 新增

- 全域 `--format <human|shasum|bsd|json|csv|ndjson>`：統一的機器可解析輸出，
  stdout 只放資料、stderr 放進度與訊息。
- `verify --report <PATH>`：輸出完整 JSON 驗證報告（含 `summary` 與 `exit_code`）。
- `verify --ignore-missing`：忽略遺失檔案，不計入錯誤。
- `hash --digest-only`：只輸出雜湊值，方便腳本取用。
- `hash --strict`、`verify --strict`：遇錯立即中止（預設為繼續並彙總）。
- `check` 子命令：驗證單一檔案或 `--text` 內容的雜湊，支援多個 `--expect` 與 `--ignore-case`。
- `diff` 子命令：比對兩份校驗清單，支援 human / csv / json 輸出。
- 新增 `report` 與 `manifest` 模組，輸出契約與清單解析／比對可直接作為 library API 使用。
- `--no-color` 與 `NO_COLOR` 環境變數支援。

### 變更

- 退出碼統一為四段式語意：`0` 成功、`1` 驗證失敗或有差異、`2` I/O 錯誤、
  `3` 用法或輸入錯誤、`4` 無可處理項目。
- `verify` 先嚴格驗證 manifest 格式；格式錯誤或無有效項目回應 `3`。
- `hash` 的讀取緩衝區限制在 4 KiB..=64 MiB；BLAKE3 整檔載入上限預設 512 MiB，
  可用環境變數 `ONEE_BLAKE3_BULK_MAX_MB` 調整（`0` 停用整檔載入）。
- `verify` JSON 報告與 `json`／`txt` 輸出的版本欄位改為套件版本（不再硬編碼）。

### 修正

- BLAKE3 對大檔案可能一次配置到 1 GiB 的記憶體峰值問題。
- `json` 輸出中 container 的 `version` 原本固定為 `0.1.0`。

### 測試與 CI

- 單元測試 55、整合測試 32、doc 測試 2，全數通過。
- CI 新增 `cargo doc --no-deps`，並將 rustdoc 警告視為錯誤。

## [2.0.3] - 2026-10-20

- 修正文件內容並加入 MSRV（1.87）驗證。
- 相容新版 clippy 的 `assert_is_empty` 檢查。
