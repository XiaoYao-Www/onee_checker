# 安全性設計文件

## 威脅模型

Onee Checker 接受兩種不受信任的外部輸入：

1. **CLI 參數** — 使用者指定的路徑、演算法、參數值
2. **Hash 驗證檔** — 第三方提供的 `*.sha256` / `*.md5` 等檔案，包含檔案路徑與預期 hash 值

攻擊面包括：Path Traversal、DoS、Hash 偽造、Parser 注入。

---

## 防護層次

```
外部輸入
    │
    ▼
┌─────────────────────────────────────┐
│  Layer 1: CLI 參數驗證              │
│  clap 型別系統 + BufferSize parser  │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Layer 2: Hash 檔解析              │
│  HashEntry 建構驗證 + 行數上限      │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Layer 3: Path Traversal 防護       │
│  sanitize_rel_path + canonicalize   │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Layer 4: I/O 邊界檢查              │
│  FileNode 深度限制 + symlink 防護   │
└─────────────────────────────────────┘
```

---

## Layer 1: CLI 參數驗證

### BufferSize parser

```rust
pub struct BufferSize(pub usize);

impl FromStr for BufferSize { ... }
```

- 僅接受數字 + K/M/G 後綴
- 拒絕空字串、負數、非數字輸入
- 最大值受 `usize` 限制

### Clap 型別系統

- `PathBuf` — 自動處理 shell glob 展開後的原始路徑
- `HashAlgo` — 僅接受枚舉值，拒絕任意字串
- `u16` (length) — 自動範圍檢查

---

## Layer 2: Hash 檔解析 (`HashEntry`)

### 行數上限 (DoS 防護)

```rust
const MAX_HASH_LINES: usize = 10_000_000;

content.lines().take(MAX_HASH_LINES)
```

### Hex 格式驗證

```rust
if !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
    return Err(HashFileParseError { ... });
}
```

- 僅接受 `0-9a-fA-F`
- 拒絕空白、特殊字元、不可見字元
- 格式錯誤的行被跳過，不中斷整體驗證

### Null byte 防護

```rust
if path.contains('\0') {
    return Err(HashFileParseError { ... });
}
```

- C 字串截斷攻擊：`file.txt\0../../etc/passwd` → 在 C 底層被截斷為 `file.txt`
- Rust 不會截斷，但防範於未然

---

## Layer 3: Path Traversal 防護 (`path_safe.rs`)

### `sanitize_rel_path(rel_path: &str, root: &Path) -> Result<PathBuf>`

執行六層檢查：

| # | 檢查 | 拒絕範例 |
|---|---|---|
| 1 | 空路徑 | `""`, `"  "` |
| 2 | Null byte | `"file\0.txt"` |
| 3 | 絕對路徑 (Unix) | `"/etc/passwd"` |
| 4 | 絕對路徑 (Windows) | `"C:\\Windows\\..."`, `"\\\\server\\share"` |
| 5 | `..` 元件 | `"../../etc/passwd"`, `"sub/../../etc"` |
| 6 | 邊界驗證 | 組合後 canonicalize，確認在 root 內 |

### `canonicalize_root(root: &Path) -> Result<PathBuf>`

- 將輸入的根目錄正規化為絕對路徑
- 追蹤 symlink
- 拒絕非目錄路徑

### Windows 裝置名稱防護

```rust
#[cfg(target_os = "windows")]
{
    if lower.contains("con") || lower.contains("nul") || ... {
        return Err(InvalidPath(...));
    }
}
```

拒絕 `CON`, `NUL`, `PRN`, `AUX`, `COM1-9`, `LPT1-9`。

---

## Layer 4: I/O 邊界檢查 (`walker.rs`)

### 遞迴深度限制

```rust
const MAX_DEPTH: u16 = 1024;

fn build_node_recursive(path: &Path, root: &Path, depth: u16) -> ... {
    if should_recurse && depth < MAX_DEPTH { ... }
}
```

- 防止極深巢狀目錄造成 stack overflow
- 超過 1024 層自動停止遞迴

### Symlink 逃脫防護

```rust
if is_symlink {
    let abs_target = if target.is_absolute() { target }
                     else { parent.join(&target) };
    match fs::canonicalize(&abs_target) {
        Ok(canon_target) => canon_target.starts_with(root),
        Err(_) => false,
    }
}
```

- 僅當 symlink 目標仍在根目錄內時才進入
- 相對 symlink 正確解析相對於 symlink 所在目錄
- 無法解析的 symlink 不進入

---

## 其他安全考量

### 記憶體安全

- **無 `unsafe` 程式碼** — 整個 codebase 不包含 `unsafe` 區塊
- **HasherEnum 參數限制** — `u16` 限制可變長度輸出 ≤ 65535 bytes（遠低於任一 alloc 失敗門檻）
- **rayon 全域池** — `map_init` 建立 thread-local buffer，無共享可變狀態

### 錯誤處理

- 所有錯誤透過 `OneeError` 枚舉處理，不在 `unwrap()` 或 `expect()` 上 panic
- `parse_hash_file` 中的錯誤被記錄但不中斷流程（graceful degradation）

### Exit code 合規

| Code | POSIX 語意 | Onee Checker 對應 |
|---|---|---|
| 0 | Success | 全部匹配 |
| 1 | General error | Hash 不匹配 |
| 2 | Misuse of shell builtins | I/O 錯誤 |
| 3 | (自訂) | 使用者輸入錯誤 |

---

## 已知限制與改善方向

1. **Hash 長度未驗證對應演算法** — `HashEntry` 不檢查 hex 長度是否與演算法輸出長度匹配（128 hex chars 的 MD5 行仍被接受）
2. **rayon 全域池僅首次生效** — 同程序內多次變更 `--threads` 無效
3. **無權限檢查** — 不強制要求讀取權限（`File::open` 失敗時由 OS 報錯）
4. **無資源限制 (ulimit)** — 不自行限制開啟檔案數量

### 對策

- 限制 1 可透過在 `HashEntry` 建構時增加可選的 `expected_len` 參數解決
- 限制 2 可在未來改用局部 `rayon::ThreadPool` 代替 `build_global()`
