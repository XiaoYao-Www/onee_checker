# 依賴授權審計報告

**專案**: onee_checker v2.0.0  
**專案授權**: MIT (with trademark retention)  
**審計日期**: 2026-06-02  

---

## 直接依賴 (17)

| 依賴 | 版本 | 授權 | SPDX | 與 MIT 相容 | 需署名 | 處理方式 |
|---|---|---|---|---|---|---|
| **clap** | 4.6.1 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **indicatif** | 0.18.4 | MIT | MIT | ✅ | ❌ | 無需操作 |
| **console** | 0.16.3 | MIT | MIT | ✅ | ❌ | 無需操作 |
| **thiserror** | 2 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **anyhow** | 1 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **md-5** | 0.11.0 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **sha-1** | 0.10.1 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **sha2** | 0.11.0 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **sha3** | 0.11.0 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **blake2** | 0.10.6 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **blake3** | 1.8.5 | CC0-1.0 OR Apache-2.0 | CC0-1.0 / Apache-2.0 | ✅ | ✅ | NOTICE 中註明 BLAKE3 團隊版權 |
| **serde** | 1.0.228 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **serde_json** | 1.0.149 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **walkdir** | 2.5.0 | MIT OR Unlicense | MIT / Unlicense | ✅ | ❌ | 無需操作 |
| **rayon** | 1.12.0 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **hex** | 0.4.3 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |
| **chrono** | 0.4.44 | MIT OR Apache-2.0 | MIT / Apache-2.0 | ✅ | ❌ | 無需操作 |

---

## 授權分布圖

```
MIT OR Apache-2.0  ████████████████  14 (82%)
CC0-1.0            ██                 1  (6%)
MIT                ██                 2 (12%)
```

---

## 傳遞依賴審計摘要

以下為 Cargo 解析出的關鍵傳遞依賴及授權：

| 傳遞依賴 | 授權 | 相容 |
|---|---|---|
| `proc-macro2` | MIT OR Apache-2.0 | ✅ |
| `quote` | MIT OR Apache-2.0 | ✅ |
| `syn` | MIT OR Apache-2.0 | ✅ |
| `unicode-ident` | MIT OR Apache-2.0 OR Unicode-3.0 | ✅ |
| `libc` | MIT OR Apache-2.0 | ✅ |
| `crossbeam-*` | MIT OR Apache-2.0 | ✅ |
| `regex-*` | MIT OR Apache-2.0 | ✅ |
| `memchr` | MIT OR Unlicense | ✅ |
| `digest` | MIT OR Apache-2.0 | ✅ |
| `crypto-common` | MIT OR Apache-2.0 | ✅ |
| `block-buffer` | MIT OR Apache-2.0 | ✅ |
| `cpufeatures` | MIT OR Apache-2.0 | ✅ |
| `keccak` | CC0-1.0 | ✅ |
| `generic-array` | MIT | ✅ |
| `typenum` | MIT OR Apache-2.0 | ✅ |
| `arrayref` | BSD-2-Clause | ✅ |
| `constant_time_eq` | CC0-1.0 | ✅ |

---

## 風險評估

### ✅ 無風險

- **無 GPL / AGPL / LGPL 相依** — 整個依賴樹不含任何 copyleft 授權
- **無 MPL / CDDL / EPL** — 無弱 copyleft
- **所有授權均與 MIT 相容** — 無衝突

### ⚠️ 需注意

| 依賴 | 注意事項 |
|---|---|
| **blake3** | CC0-1.0 在部分司法管轄區（如德國、法國）不承認公共領域放棄。建議在 NOTICE 中保留 BLAKE3 團隊的版權聲明 |
| **keccak** (sha3 傳遞依賴) | 同為 CC0-1.0，無需額外操作 |
| **arrayref** | BSD-2-Clause 要求在散佈時保留版權聲明及免責聲明。作為編譯時靜態連結的二進位，無需額外操作 |

### 已執行的合規操作

| 操作 | 狀態 |
|---|---|
| ✅ LICENSE 改為 MIT | 完成 |
| ✅ NOTICE 含商標保留 + RustCrypto/BLAKE3 署名 | 完成 |
| ✅ Cargo.toml `license = "MIT"` | 已正確 |
| ✅ README License 章節更新 | 完成 |

---

## 商標說明

本專案的 MIT 授權僅涵蓋程式碼的著作權。以下為作者保留的商標權：

| 名稱 | 類型 | 限制 |
|---|---|---|
| "Onee Checker" | 專案名稱 | 衍生作品須更名 |
| "oneechk" | 二進位名稱 | 分支/衍生二進位須更名 |

任意衍生作品或分支必須：
1. 使用不同的專案名稱和二進位名稱
2. 在 README 中清楚標示為衍生作品
3. 保留原始作者 (逍遙 / XiaoYao) 的署名（如 LICENSE 及 NOTICE 所載）

---

## 參考資料

- [SPDX License List](https://spdx.org/licenses/)
- [RustCrypto Licenses](https://github.com/RustCrypto)
- [BLAKE3 License](https://github.com/BLAKE3-team/BLAKE3/blob/master/LICENSE_Apache2)
- [MIT License FAQ](https://opensource.org/license/mit)
- [CC0 FAQ](https://wiki.creativecommons.org/wiki/CC0_FAQ)
