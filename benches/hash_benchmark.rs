// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Onee Checker 基準測試套件
//!
//! 使用 criterion 框架量測 hash 演算法效能、hex 編碼方法比較等。
//!
//! 執行方式：
//! ```bash
//! cargo bench
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// ──────────────────────────────────────────────────────────
//  單一演算法基準
// ──────────────────────────────────────────────────────────

/// SHA-256: 1MB 資料
fn bench_sha256_1mb(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024];
    c.bench_function("sha256_1mb", |b| {
        b.iter(|| {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(black_box(&data));
            let _ = hasher.finalize();
        });
    });
}

/// BLAKE3: 1MB 資料
fn bench_blake3_1mb(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024];
    c.bench_function("blake3_1mb", |b| {
        b.iter(|| {
            let mut hasher = blake3::Hasher::new();
            hasher.update(black_box(&data));
            let _ = hasher.finalize();
        });
    });
}

/// BLAKE3: 100MB 資料（串流 — 單線程 update 模式）
fn bench_blake3_100mb(c: &mut Criterion) {
    let data = vec![0u8; 100 * 1024 * 1024];
    c.bench_function("blake3_100mb_stream", |b| {
        b.iter(|| {
            let mut hasher = blake3::Hasher::new();
            // 模擬串流：分 100 次 update
            for chunk in black_box(&data).chunks(1024 * 1024) {
                hasher.update(chunk);
            }
            let _ = hasher.finalize();
        });
    });
}

// ──────────────────────────────────────────────────────────
//  多演算法基準（單次 I/O vs 兩次 I/O）
// ──────────────────────────────────────────────────────────

/// 多演算法：SHA256 + BLAKE3 同時計算（一次 update）
fn bench_multi_algo_simultaneous(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024];
    c.bench_function("multi_algo_sha256_blake3_simul_1mb", |b| {
        b.iter(|| {
            use sha2::Digest;
            let mut sha = sha2::Sha256::new();
            let mut bl = blake3::Hasher::new();
            // 一次 update 同時餵入兩個 hasher
            let d = black_box(&data);
            sha.update(d);
            bl.update(d);
            let _ = sha.finalize();
            let _ = bl.finalize();
        });
    });
}

/// 多演算法：SHA256 + BLAKE3 分開計算（兩次 update）
fn bench_multi_algo_separate(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024];
    c.bench_function("multi_algo_sha256_blake3_separate_1mb", |b| {
        b.iter(|| {
            use sha2::Digest;
            let mut sha = sha2::Sha256::new();
            sha.update(black_box(&data));
            let _ = sha.finalize();
            let mut bl = blake3::Hasher::new();
            bl.update(black_box(&data));
            let _ = bl.finalize();
        });
    });
}

// ──────────────────────────────────────────────────────────
//  BLAKE3 bulk vs stream 比較
// ──────────────────────────────────────────────────────────

/// BLAKE3: bulk hash（blake3::hash 內部多線程）
fn bench_blake3_bulk_100mb(c: &mut Criterion) {
    let data = vec![0u8; 100 * 1024 * 1024];
    c.bench_function("blake3_100mb_bulk", |b| {
        b.iter(|| {
            let result = blake3::hash(black_box(&data));
            black_box(result);
        });
    });
}

// ──────────────────────────────────────────────────────────
//  hex 編碼方法比較
// ──────────────────────────────────────────────────────────

/// hex 編碼方法比較：逐 byte `write!` vs `hex::encode`
fn bench_hex_encode(c: &mut Criterion) {
    let bytes = vec![0xabu8; 32];
    let mut group = c.benchmark_group("hex_encode");

    group.bench_function("write_macro", |b| {
        b.iter(|| {
            let mut buf = String::new();
            buf.reserve(bytes.len() * 2);
            for byte in black_box(&bytes) {
                use std::fmt::Write;
                let _ = write!(buf, "{byte:02x}");
            }
            black_box(buf);
        });
    });

    group.bench_function("hex_encode", |b| {
        b.iter(|| {
            let result = hex::encode(black_box(&bytes));
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sha256_1mb,
    bench_blake3_1mb,
    bench_blake3_100mb,
    bench_blake3_bulk_100mb,
    bench_multi_algo_simultaneous,
    bench_multi_algo_separate,
    bench_hex_encode,
);
criterion_main!(benches);
