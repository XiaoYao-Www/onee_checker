# Onee Checker 全面檢視與修正計劃

**檢視日期：** 2026-10-20  
**範圍：** CLI、hash 引擎、檔案系統 walker/parser、文件與 CI 流程

## 目標

讓 `oneechk` 對腳本和 CI 提供可依賴的退出碼與輸出，確保部分失敗不會看起來像成功；保持 library API 的相容性，並讓文件與實際行為一致。

## 專案現況

- 專案採 library-first 架構：`src/lib.rs` 提供 API，`src/main.rs` 負責 Clap 解析、子命令調度與退出。
- CLI 提供 `hash`、`verify`、`json`、`txt`；Hash 支援多演算法與 Rayon 並行。
- `OneeError::exit_code()` 已定義 `0` 成功、`1` hash 不匹配、`2` I/O/序列化、`3` 輸入或格式錯誤。
- CI 已涵蓋 Windows、Linux、macOS 的測試、Clippy、格式檢查，另有安全與授權檢查。
- 文件已有 CLI、API、架構、安全、開發指引；CLI 文件原先宣稱的部分失敗語意未被程式落實。

## 檢視發現與優先級

| 優先級 | 問題 | 影響 |
|---|---|---|
| P0 | `hash` 將逐檔錯誤印出後仍回傳成功，且可能寫出部分 checksum | CI 或腳本收到 exit code 0，誤以為完整清單已產生 |
| P0 | `verify` 將逐檔 I/O/path 錯誤計數，卻只對 hash 不匹配回傳錯誤 | 缺檔或無法讀取時仍可能回傳 exit code 0 |
| P0 | hash 檔 parser 對格式錯誤行採跳過策略，空或全損壞清單也可能驗證成功 | 損壞清單被當成有效驗證結果 |
| P1 | `list_files` 忽略 `walkdir` 遍歷錯誤 | 權限受限子目錄可能被漏掉，checksum 清單不完整但成功 |
| P1 | 預設 hash 檔寫在目前工作目錄，記錄路徑卻以來源父目錄為根 | 從不同目錄執行生成與驗證會找不到記錄中的檔案 |
| P1 | Clap 參數錯誤使用框架預設退出碼，和文件定義不一致 | 呼叫端無法依專案退出碼契約分類錯誤 |
| P2 | 多演算法進度條每演算法都累加，但總量只設定檔案數 | 進度值超出總量 |

## 設計決策

### CLI 退出碼

| 代碼 | 意義 |
|---|---|
| 0 | 成功；`--help`、`--version` 也屬成功 |
| 1 | checksum 不匹配 |
| 2 | I/O、序列化或執行期間錯誤；hash 部分失敗也屬此類 |
| 3 | CLI 參數、路徑或 hash 清單格式錯誤 |

多項驗證同時失敗時，I/O/路徑錯誤優先回傳 `2`，否則有 hash 不匹配回傳 `1`。Clap 的語法/值解析錯誤由入口轉成 `3`；help/version 保持 `0`。

### 輸出與完整性

- stdout 僅供命令資料；狀態、進度與錯誤寫入 stderr。
- Hash 對任何來源檔案讀取失敗都不產生部分 checksum 輸出，並回傳 `2`。
- 預設 checksum 檔放在來源旁；相對路徑基準使用來源父目錄，符合 `verify` 預設根目錄。
- CLI 使用嚴格 checksum parser；既有寬鬆 library parser 保留，避免破壞既有呼叫端。
- Walker 以錯誤中止掃描，避免漏檔後繼續生成看似完整的結果。

## 已完成變更

- 統一 Clap 參數錯誤與應用程式錯誤退出碼，保留 help/version 成功狀態。
- 修正單演算法與多演算法 hash 將逐檔錯誤誤當成功的問題；有錯時不寫部分結果。
- 修正 verify 對逐檔錯誤回傳成功的問題，並依既定退出碼優先序回報。
- 新增嚴格 parser，拒絕壞行、過量記錄與空清單；保留舊 parser API。
- Walker 不再靜默丟棄遍歷錯誤。
- 預設 checksum 檔位置改為來源旁；修正多演算法進度累加。
- 更新 README、CLI、架構與開發文件，記錄修正後的契約。

## 建議的後續驗收

1. 執行格式檢查、Clippy 與跨平台 CI。
2. 以暫存目錄驗收：單/多演算法成功往返、檔案修改、缺檔、拒絕讀取、損壞/空 checksum、錯誤參數、help/version、stdout/stderr 分流。
3. 確認 Linux、Windows、macOS 的路徑分隔與預設輸出位置行為一致。
4. 審視 README/API 對既有 `parse_hash_file` 寬鬆行為的說明，視需要在下一個 minor release 再考慮棄用策略。
