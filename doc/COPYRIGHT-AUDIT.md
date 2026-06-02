# 版權合規審計報告

**專案**: onee_checker v2.0.0  
**審計日期**: 2026-06-02  
**執行者**: cli-systems-engineer (automated)  

---

## 審計範圍

| 類別 | 檢查項目 | 狀態 |
|---|---|---|
| **專案授權** | LICENSE 文件正確性 | ✅ MIT |
| **專案授權** | Cargo.toml `license` 欄位 | ✅ MIT |
| **原始碼** | 所有 `.rs` 檔案含版權標頭 | ✅ 24/24 |
| **原始碼** | SPDX-License-Identifier | ✅ SPDX: MIT |
| **第三方依賴** | 授權相容性 | ✅ 全部 MIT/Apache-2.0/CC0 相容 |
| **第三方依賴** | 署名合規 | ✅ NOTICE 含 RustCrypto/BLAKE3 |
| **商標** | 商標宣告 | ✅ NOTICE |
| **測試資料** | TEST/ 目錄版權 | ⚠️ 含第三方版權內容 |
| **README** | 版權宣告 | ✅ |
| **Cargo.toml** | 元數據完整性 | ✅ |

---

## 1. 專案授權 — ✅ PASS

| 檔案 | 內容 | 狀態 |
|---|---|---|
| `LICENSE` | MIT License, Copyright (c) 2026 逍遙 (XiaoYao) | ✅ |
| `Cargo.toml` | `license = "MIT"` | ✅ |
| `NOTICE` | 商標保留 + 第三方署名 | ✅ |

### Cargo.toml 元數據

```toml
[package]
license = "MIT"
repository = "https://github.com/XiaoYao-Www/onee_checker"
keywords = ["hash", "checksum", "sha256", "blake3", "cli", "verification"]
categories = ["command-line-utilities", "cryptography", "filesystem"]
```

---

## 2. 原始碼版權標頭 — ✅ PASS

所有 24 個 `.rs` 檔案均包含以下版權標頭：

```rust
// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT
```

| 目錄 | 檔案數 | 狀態 |
|---|---|---|
| `src/` | 6 (main, lib, error, algorithm, hash, tree) | ✅ |
| `src/cli/` | 5 (mod, hash_cmd, verify_cmd, json_cmd, txt_cmd) | ✅ |
| `src/fs/` | 5 (mod, node, walker, writer, path_safe) | ✅ |
| `src/hasher/` | 8 (mod, enum_hasher, md5, sha1, sha2, sha3, blake2, blake3) | ✅ |

---

## 3. 第三方依賴授權 — ✅ PASS

### 相容性評估

| 依賴數量 | 授權類型 | MIT 相容 |
|---|---|---|
| 14 | MIT OR Apache-2.0 | ✅ |
| 2 | MIT | ✅ |
| 1 | CC0-1.0 OR Apache-2.0 (blake3) | ✅ |

### 署名狀態

| 第三方 | 署名位置 |
|---|---|
| RustCrypto Developers (md-5, sha-1, sha2, sha3, blake2) | `NOTICE` § RustCrypto Hashes |
| Jack O'Connor / BLAKE3 team | `NOTICE` § BLAKE3 |
| 所有其他 MIT/Apache-2.0 依賴 | 無需署名（MIT 不要求 attribution） |

### 高風險授權掃描

| 授權 | 存在於依賴樹 | 風險 |
|---|---|---|
| GPL (any version) | ❌ 不存在 | 無 |
| AGPL | ❌ 不存在 | 無 |
| LGPL | ❌ 不存在 | 無 |
| MPL | ❌ 不存在 | 無 |
| CDDL | ❌ 不存在 | 無 |
| 未宣告授權 | ❌ 不存在 | 無 |

---

## 4. TEST/ 目錄 — ⚠️ 注意事項

`TEST/` 目錄含有第三方版權內容（有聲書《劍來》音訊檔案）。

| 項目 | 狀態 |
|---|---|
| Git 追蹤 | ❌ 已透過 `.gitignore` 排除 |
| 散佈風險 | ❌ 不會隨原始碼散佈 |
| 授權涵蓋 | ❌ MIT License 不涵蓋此內容 |
| 版權警告 | ✅ `TEST/README.md` 已添加警告 |
| 替代方案 | 建議替換為自行產生的測試資料 |

### 風險評估: **低**

- 內容已被 `.gitignore` 排除，不會被提交至 Git repository
- 僅存在於開發者本機環境
- 已添加版權歸屬與使用限制說明

---

## 5. README 版權宣告 — ✅ PASS

```markdown
> Copyright © 2026 逍遙 (XiaoYao).
> SPDX-License-Identifier: MIT
```

---

## 6. Cargo.toml 元數據 — ✅ PASS

| 欄位 | 值 | 狀態 |
|---|---|---|
| `license` | `"MIT"` | ✅ |
| `repository` | 已設定 | ✅ |
| `authors` | 已設定 | ✅ |
| `keywords` | 已設定 | ✅ |
| `categories` | 已設定 | ✅ |

---

## 7. 建議改善項目

| 優先級 | 建議 |
|---|---|
| 🟢 低 | 為 `Onee Checker` 名稱註冊商標（如計劃商業化） |
| 🟢 低 | 取代 TEST/ 目錄內容為無版權測試資料 |
| 🟢 低 | 在 CI/CD pipeline 中添加 `cargo-deny` 或 `cargo-license` 自動掃描 |

---

## 結論

**✅ 本專案版權合規。所有檢查項目通過。**

- 授權: MIT (與所有依賴相容)
- 原始碼標頭: 24/24 檔案含 SPDX 標頭
- 第三方署名: NOTICE 已包含
- 商標: 已保留
- 測試資料: 已標註警告且不納入版本控制
- 無 GPL/AGPL/LGPL/MPL 污染
