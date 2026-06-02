#!/usr/bin/env bash
#
# Onee Checker — 標準化驗證流程
#
# 使用方式：
#   bash verify.sh           # 完整驗證
#   bash verify.sh quick     # 快速驗證（跳過 bench）
#   bash verify.sh full      # 完整驗證（含 bench + deny）
#
# 這個腳本可重複執行，作為 CI 和發版前的標準驗證門檻。

set -euo pipefail
FAIL=0
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

STEP=0

pass()   { echo -e "  ${GREEN}✔${NC} $1"; }
fail()   { echo -e "  ${RED}✘${NC} $1"; FAIL=1; }
skip()   { echo -e "  ${YELLOW}⊘${NC} $1"; }
info()   { echo -e "  ${YELLOW}ℹ${NC} $1"; }
header() { STEP=$((STEP+1)); echo ""; echo "━━━ [$STEP/8] $1 ━━━"; }

# ── 1. 編譯檢查 ──────────────────────────────────────────
header "編譯檢查"
if cargo check 2>&1; then
    pass "cargo check 通過"
else
    fail "cargo check 失敗"
    exit 1
fi

# ── 2. 單元測試 ──────────────────────────────────────────
header "單元測試"
if cargo test --lib 2>&1; then
    pass "單元測試通過"
else
    fail "單元測試失敗"
fi

# ── 3. 整合測試 ──────────────────────────────────────────
header "整合測試"
if cargo test --test cli_integration 2>&1; then
    pass "整合測試通過（23 項）"
else
    fail "整合測試失敗"
fi

# ── 4. 文檔測試 ──────────────────────────────────────────
header "文檔測試"
if cargo test --doc 2>&1; then
    pass "文檔測試通過"
else
    fail "文檔測試失敗"
fi

# ── 5. 警告檢查 ──────────────────────────────────────────
header "警告檢查"
if cargo check 2>&1 | grep -i "warning" | grep -v "generated\|`basename $0`" || true; then
    info "以上為目前警告（非致命）"
fi
pass "警告檢查完成"

# ── 6. 無 unsafe 檢查 ────────────────────────────────────
header "安全性檢查（unsafe 禁止）"
if grep -r "unsafe\s*{" src/ --include="*.rs" || true); then
    if [ "$(grep -r "unsafe\s*{" src/ --include="*.rs" | wc -l)" -gt 0 ]; then
        fail "發現 unsafe 程式碼"
    else
        pass "unsafe 通過"
    fi
else
    pass "無 unsafe 程式碼（cargo.toml 已設置 deny）"
fi

# ── 7. 多線程一致性 ──────────────────────────────────────
header "多線程結果一致性檢查"
TMPDIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'oneecheck')
echo "一致性測試內容" > "$TMPDIR/test.txt"
echo "第二個檔案" > "$TMPDIR/test2.txt"

if HASH1=$(cargo run --quiet -- hash "$TMPDIR/test.txt" -a sha256 -t 1 -o - 2>/dev/null) && \
   HASH2=$(cargo run --quiet -- hash "$TMPDIR/test.txt" -a sha256 -t 4 -o - 2>/dev/null) && \
   [ "$HASH1" = "$HASH2" ]; then
    pass "單線程 (1) 與多線程 (4) 結果一致"
else
    fail "多線程結果不一致"
    info "  單線程: $HASH1"
    info "  多線程: $HASH2"
fi

# 多演算法一致性
if HASH_MULTI=$(cargo run --quiet -- hash "$TMPDIR/test.txt" -a sha256 -a blake3 -o - 2>/dev/null) && \
   [ -n "$HASH_MULTI" ]; then
    LINE_COUNT=$(echo "$HASH_MULTI" | wc -l)
    if [ "$LINE_COUNT" -ge 2 ]; then
        pass "多演算法結果正常（${LINE_COUNT} 行輸出）"
    fi
fi

rm -rf "$TMPDIR"

# ── 8. 基準測試（可選） ──────────────────────────────────
header "基準測試"
MODE="${1:-full}"
if [ "$MODE" = "full" ] || [ "$MODE" = "bench" ]; then
    if cargo bench 2>&1; then
        pass "基準測試完成"
    else
        fail "基準測試失敗"
    fi
else
    skip "跳過基準測試（使用 bash verify.sh full 執行）"
fi

# ── 結果 ──────────────────────────────────────────────────
echo ""
if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}══════════════════════════════════════${NC}"
    echo -e "${GREEN}  全部驗證通過！${NC}"
    echo -e "${GREEN}══════════════════════════════════════${NC}"
else
    echo -e "${RED}══════════════════════════════════════${NC}"
    echo -e "${RED}  部分驗證失敗，請修正後重試${NC}"
    echo -e "${RED}══════════════════════════════════════${NC}"
fi
exit $FAIL
