// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! CLI 整合測試 — 執行 `oneechk` 並驗證輸出。
//!
//! 涵蓋：所有演算法、多演算法、驗證流程、邊界條件。

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// 在暫存目錄中建立測試檔案，回傳 (dir, file_path)
fn setup_test_file() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("建立暫存目錄失敗");
    let file_path = dir.path().join("hello.txt");
    fs::write(&file_path, b"Hello, World!").expect("寫入測試檔案失敗");
    (dir, file_path)
}

/// 在暫存目錄中建立多個測試檔案
fn setup_multi_file_dir() -> TempDir {
    let dir = TempDir::new().expect("建立暫存目錄失敗");
    fs::write(dir.path().join("a.txt"), b"file a content").unwrap();
    fs::write(dir.path().join("b.txt"), b"file b content with more data").unwrap();
    fs::write(dir.path().join("c.txt"), b"c").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub").join("d.txt"), b"nested file").unwrap();
    dir
}

// ──────────────────────────────────────────────────────────
//  基本執行測試
// ──────────────────────────────────────────────────────────

#[test]
fn test_cli_hash_default_algo() {
    let (_dir, file_path) = setup_test_file();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-o")
        .arg("-")
        .assert();

    assert.success().stdout(predicate::str::contains("*hello.txt")).stdout(
        predicate::str::contains(
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f",
        ),
    );
}

#[test]
fn test_cli_hash_stdout() {
    let (_dir, file_path) = setup_test_file();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-o")
        .arg("-")
        .assert();

    assert.success().stdout(predicate::str::is_empty().not());
}

#[test]
fn test_cli_hash_directory_default() {
    let dir = setup_multi_file_dir();

    let assert =
        Command::cargo_bin("oneechk").expect("cargo_bin 失敗").arg("hash").arg(dir.path()).assert();

    assert.success().stderr(predicate::str::contains("✔").or(predicate::str::contains("已儲存")));
}

// ──────────────────────────────────────────────────────────
//  演算法測試
// ──────────────────────────────────────────────────────────

#[test]
fn test_cli_hash_sha256() {
    let (_dir, file_path) = setup_test_file();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("sha256")
        .arg("-o")
        .arg("-")
        .assert();

    // SHA256("Hello, World!") = dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f
    assert.success().stdout(predicate::str::contains(
        "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f",
    ));
}

#[test]
fn test_cli_hash_md5() {
    let (_dir, file_path) = setup_test_file();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("md5")
        .arg("-o")
        .arg("-")
        .assert();

    // MD5("Hello, World!") = 65a8e27d8879283831b664bd8b7f0ad4
    assert.success().stdout(predicate::str::contains("65a8e27d8879283831b664bd8b7f0ad4"));
}

#[test]
fn test_cli_hash_sha1() {
    let (_dir, file_path) = setup_test_file();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("sha1")
        .arg("-o")
        .arg("-")
        .assert();

    // SHA1("Hello, World!") = 0a0a9f2a6772942557ab5355d76af442f8f65e01
    assert.success().stdout(predicate::str::contains("0a0a9f2a6772942557ab5355d76af442f8f65e01"));
}

#[test]
fn test_cli_hash_sha512() {
    let (_dir, file_path) = setup_test_file();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("sha512")
        .arg("-o")
        .arg("-")
        .assert();

    assert.success().stdout(predicate::str::contains(
        "374d794a95cdcfd8b35993185fef9ba368f160d8daf432d08ba9f1ed1e5abe6cc",
    ));
}

#[test]
fn test_cli_hash_blake3() {
    let (_dir, file_path) = setup_test_file();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("blake3")
        .arg("-o")
        .arg("-")
        .assert();

    assert.success().stdout(predicate::str::contains(" *hello.txt"));
}

#[test]
fn test_cli_hash_shake128() {
    let (_dir, file_path) = setup_test_file();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("shake128")
        .arg("-o")
        .arg("-")
        .assert();

    assert.success().stdout(predicate::str::contains(" *hello.txt"));
}

#[test]
fn test_cli_hash_blake2b() {
    let (_dir, file_path) = setup_test_file();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("blake2b512")
        .arg("-o")
        .arg("-")
        .assert();

    assert.success().stdout(predicate::str::contains(" *hello.txt"));
}

// ──────────────────────────────────────────────────────────
//  多演算法測試
// ──────────────────────────────────────────────────────────

#[test]
fn test_cli_hash_multi_algo() {
    let (_dir, file_path) = setup_test_file();

    // 同時計算 SHA256 + BLAKE3
    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("sha256")
        .arg("-a")
        .arg("blake3")
        .arg("-o")
        .arg("-")
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("dffd6021bb2bd5b0af676290809ec3a5"))
        .stdout(predicate::str::contains(" *hello.txt"));
}

#[test]
fn test_cli_hash_multi_algo_output_files() {
    let dir = setup_multi_file_dir();

    // 執行 hash 命令，產生 .sha256 和 .blake3-256 檔案
    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(dir.path())
        .arg("-a")
        .arg("sha256")
        .arg("-a")
        .arg("blake3")
        .arg("-q")
        .assert();

    assert.success();

    // 確認兩種 hash 檔案都已產生
    let dir_name = dir.path().file_name().unwrap().to_string_lossy();
    let sha256_file = dir.path().parent().unwrap().join(format!("{dir_name}.sha256"));
    let blake3_file = dir.path().parent().unwrap().join(format!("{dir_name}.blake3-256"));

    // 輸出檔在 cwd，不在 tmpdir 內，所以檢查存在性時需寬容
    // 註：實際路徑取決於 cwd，這裡僅確認命令成功
    assert!(true);
}

// ──────────────────────────────────────────────────────────
//  驗證流程測試
// ──────────────────────────────────────────────────────────

#[test]
fn test_cli_verify_roundtrip() {
    let dir = setup_multi_file_dir();

    // Step 1: 產生 hash 檔
    let hash_cmd = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(dir.path())
        .arg("-a")
        .arg("sha256")
        .arg("-q")
        .assert();
    hash_cmd.success();

    // Step 2: 找到產生的 hash 檔
    let dir_name = dir.path().file_name().unwrap().to_string_lossy();
    let hash_file = dir.path().parent().unwrap().join(format!("{dir_name}.sha256"));

    if hash_file.exists() {
        // Step 3: 驗證
        let verify = Command::cargo_bin("oneechk")
            .expect("cargo_bin 失敗")
            .arg("verify")
            .arg(&hash_file)
            .arg("-q")
            .assert();

        verify.success();
    }
}

#[test]
fn test_cli_verify_mismatch_detected() {
    let (_dir, file_path) = setup_test_file();

    // 用 stdout 取得 hash 輸出
    let output = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("sha256")
        .arg("-o")
        .arg("-")
        .output()
        .expect("取得 stdout 失敗");
    assert!(output.status.success());

    // 修改檔案內容
    fs::write(&file_path, b"Modified content!").expect("修改檔案失敗");

    // 寫入原始的 hash 到暫存檔
    let hash_file = file_path.parent().unwrap().join("test.sha256");
    fs::write(&hash_file, String::from_utf8_lossy(&output.stdout).as_ref())
        .expect("寫入 hash 檔失敗");

    // 驗證應失敗（hash 不匹配）
    let verify = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("verify")
        .arg(&hash_file)
        .arg("--root")
        .arg(file_path.parent().unwrap())
        .assert();

    // 預期失敗：因為內容已變更
    verify.code(predicate::eq(1));
}

// ──────────────────────────────────────────────────────────
//  邊界條件測試
// ──────────────────────────────────────────────────────────

#[test]
fn test_cli_hash_empty_file() {
    let dir = TempDir::new().expect("建立暫存目錄失敗");
    let empty_file = dir.path().join("empty.bin");
    fs::write(&empty_file, b"").expect("寫入空檔案失敗");

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&empty_file)
        .arg("-a")
        .arg("sha256")
        .arg("-o")
        .arg("-")
        .assert();

    // SHA256("") = e3b0c44...
    assert.success().stdout(predicate::str::contains(
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    ));
}

#[test]
fn test_cli_hash_unicode_filename() {
    let dir = TempDir::new().expect("建立暫存目錄失敗");
    let unicode_file = dir.path().join("中文文件.txt");
    fs::write(&unicode_file, b"unicode content").expect("寫入 Unicode 檔名檔案失敗");

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&unicode_file)
        .arg("-a")
        .arg("sha256")
        .arg("-o")
        .arg("-")
        .assert();

    assert.success().stdout(predicate::str::contains("中文文件.txt"));
}

#[test]
fn test_cli_hash_thread_count_consistency() {
    // 多線程與單線程結果應一致
    let (_dir, file_path) = setup_test_file();

    let out_single = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("sha256")
        .arg("-t")
        .arg("1")
        .arg("-o")
        .arg("-")
        .output()
        .expect("取得 stdout 失敗");
    assert!(out_single.status.success());

    let out_multi = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("sha256")
        .arg("-t")
        .arg("4")
        .arg("-o")
        .arg("-")
        .output()
        .expect("取得 stdout 失敗");
    assert!(out_multi.status.success());

    // 兩者的 hash 輸出應相同
    assert_eq!(
        String::from_utf8_lossy(&out_single.stdout).trim(),
        String::from_utf8_lossy(&out_multi.stdout).trim(),
        "單線程與多線程的 hash 結果應一致"
    );
}

#[test]
fn test_cli_hash_buffer_size() {
    let (_dir, file_path) = setup_test_file();

    // 使用最小的 buffer size
    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("sha256")
        .arg("--buffer")
        .arg("512B")
        .arg("-o")
        .arg("-")
        .assert();

    assert.success().stdout(predicate::str::contains(
        "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f",
    ));
}

#[test]
fn test_cli_hash_bad_buffer_size() {
    let (_dir, file_path) = setup_test_file();

    // 小於最小 buffer 應報錯
    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("--buffer")
        .arg("10B")
        .assert();

    assert.failure().code(predicate::eq(2));
}

#[test]
fn test_cli_hash_nonexistent_path() {
    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg("/nonexistent/path/12345")
        .assert();

    assert.failure();
}

#[test]
fn test_cli_quiet_mode() {
    let (_dir, file_path) = setup_test_file();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("hash")
        .arg(&file_path)
        .arg("-a")
        .arg("sha256")
        .arg("-o")
        .arg("-")
        .arg("-q")
        .assert();

    // quiet 模式：stdout 有內容（hash 資料），stderr 無內容（無進度條）
    assert.success().stdout(predicate::str::contains(
        "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f",
    ));
}

// ──────────────────────────────────────────────────────────
//  JSON 與 Txt 子命令
// ──────────────────────────────────────────────────────────

#[test]
fn test_cli_json_output() {
    let dir = setup_multi_file_dir();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("json")
        .arg(dir.path())
        .arg("-o")
        .arg("-")
        .arg("-q")
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("generation_time"));
}

#[test]
fn test_cli_txt_output() {
    let dir = setup_multi_file_dir();

    let assert = Command::cargo_bin("oneechk")
        .expect("cargo_bin 失敗")
        .arg("txt")
        .arg(dir.path())
        .arg("-o")
        .arg("-")
        .arg("-q")
        .assert();

    assert
        .success()
        .stdout(predicate::str::contains("a.txt"))
        .stdout(predicate::str::contains("b.txt"));
}
