// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Onee Checker CLI — 薄調度層，業務邏輯全在 library。
//!
//! 設計原則：
//! - stdout → 機器可解析的資料（hash 檔內容、JSON、純文字樹）
//! - stderr → 人類可讀的訊息（進度、錯誤、狀態）
//! - exit code → 0=成功 1=hash不匹配 2=I/O錯誤 3=使用者錯誤

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write, stdout};
use std::path::Path;
use std::path::PathBuf;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::ThreadPoolBuilder;

use onee_checker::algorithm::{HashAlgo, HashData, HashType};
use onee_checker::cli::{self, Cli, Commands, SizeType};
use onee_checker::error::OneeError;
use onee_checker::error::Result;
use onee_checker::fs::{list_files, save_hash_file, validate_path, FileNodeContainer};
use onee_checker::hash::compute_hashes_parallel;
use onee_checker::hash::verify_hash_file;
use onee_checker::tree::{write_tree, SizeFormat, TreeOption};

fn main() {
    let cli = <Cli as clap::Parser>::parse();

    let result = match cli.command {
        Commands::Hash(args) => cmd_hash(args),
        Commands::Verify(args) => cmd_verify(args),
        Commands::Json(args) => cmd_json(args),
        Commands::Txt(args) => cmd_txt(args),
    };

    if let Err(err) = result {
        eprintln!("{} {}", style("✘").red(), err);
        std::process::exit(err.exit_code());
    }
}

// ──────────────────────────────────────────────────────────
//  hash 子命令
// ──────────────────────────────────────────────────────────

fn cmd_hash(args: cli::HashArgs) -> Result<()> {
    let path = &args.path;

    validate_path(path).map_err(|e| OneeError::InvalidPath(e))?;

    // 如果沒有指定 algo，預設使用 SHA-256
    let algos: Vec<(HashAlgo, HashType)> = if args.algo.is_empty() {
        vec![(HashAlgo::Sha256, HashType::SHA256)]
    } else {
        if args.length.is_some() {
            for algo in &args.algo {
                if !algo.can_specify_length() {
                    return Err(OneeError::UnsupportedLength {
                        algorithm: format!("{:?}", algo),
                    });
                }
            }
        }
        args.algo
            .iter()
            .map(|a: &HashAlgo| {
                a.to_hash_type(args.length)
                    .map(|ht: HashType| (a.clone(), ht))
            })
            .collect::<Result<Vec<(HashAlgo, HashType)>>>()?
    };

    // 設定線程池
    if let Some(n) = args.threads {
        ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .map_err(|e| OneeError::ArgumentError(format!("線程池設定失敗: {e}")))?;
    }

    // 收集檔案
    let files = list_files(path)?;
    if files.is_empty() {
        eprintln!("{} 警告: 指定路徑下無任何檔案", style("⚠").yellow());
        return Ok(());
    }

    let buffer_size = args.buffer.0;

    for (i, (_, hash_type)) in algos.iter().enumerate() {
        if !args.quiet && algos.len() > 1 {
            eprintln!("[{}/{}] 計算 {}...", i + 1, algos.len(), hash_type.display_name());
        }

        // 進度條
        let pb: Option<ProgressBar> = if !args.quiet {
            let bar = ProgressBar::new(files.len() as u64);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            Some(bar)
        } else {
            None
        };

        // 並行計算
        let results = compute_hashes_parallel(&files, hash_type, buffer_size);
        let mut hash_data: Vec<HashData> = Vec::with_capacity(results.len());

        for result in results {
            match result {
                Ok(data) => {
                    hash_data.push(data);
                }
                Err(e) => {
                    eprintln!("{} 錯誤: {e}", style("✘").red());
                }
            }
            if let Some(ref bar) = pb {
                bar.inc(1);
            }
        }

        if let Some(bar) = pb {
            bar.finish_and_clear();
        }

        // 排序
        hash_data.sort_by(|a, b| a.path.cmp(&b.path));

        // 決定輸出
        let current_dir = env::current_dir().map_err(OneeError::Io)?;
        let default_name = format!(
            "{}.{}",
            path.file_name().unwrap_or_default().to_string_lossy(),
            hash_type.suffix()
        );

        match &args.output {
            Some(out) if out.to_string_lossy() == "-" => {
                write_hash_to_stdout(&hash_data, path, &current_dir)?;
            }
            Some(out) => {
                let out_path: &Path = out.as_path();
                save_hash_file(&hash_data, out_path, path.parent().unwrap_or(&current_dir))?;
                eprintln!("{} 已儲存: {}", style("✔").green(), out.display());
            }
            None => {
                let output_path = current_dir.join(&default_name);
                save_hash_file(&hash_data, &output_path, path.parent().unwrap_or(&current_dir))?;
                eprintln!("{} 已儲存: {}", style("✔").green(), output_path.display());
            }
        }
    }

    Ok(())
}

/// 將 hash 結果寫入 stdout
fn write_hash_to_stdout(data: &[HashData], input_path: &Path, current_dir: &Path) -> Result<()> {
    let mut stdout = BufWriter::new(stdout().lock());
    let root = input_path.parent().unwrap_or(current_dir);
    for entry in data {
        let rel_path = entry.path.strip_prefix(root).unwrap_or(&entry.path);
        let path_str = rel_path.to_string_lossy().replace('\\', "/");
        writeln!(stdout, "{} *{}", entry.hash_hex(), path_str)
            .map_err(OneeError::Io)?;
    }
    stdout.flush().map_err(OneeError::Io)?;
    Ok(())
}

// ──────────────────────────────────────────────────────────
//  verify 子命令
// ──────────────────────────────────────────────────────────

fn cmd_verify(args: cli::VerifyArgs) -> Result<()> {
    let hashfile = &args.hashfile;

    if !hashfile.exists() || !hashfile.is_file() {
        return Err(OneeError::InvalidPath(format!(
            "Hash 檔不存在: {}",
            hashfile.display()
        )));
    }

    // 從副檔名推斷演算法
    let hash_type = match &args.algo {
        Some(algo) => algo.to_hash_type(None)?,
        None => {
            let ext = hashfile
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .ok_or_else(|| {
                    OneeError::ArgumentError("無法從副檔名推斷演算法，請使用 --algo 手動指定".into())
                })?;
            let algo = HashAlgo::from_suffix(&ext).ok_or_else(|| {
                OneeError::UnsupportedAlgorithm(format!("無法從副檔名 .{ext} 推斷演算法"))
            })?;
            algo.to_hash_type(None)?
        }
    };

    let root_dir: PathBuf = args.root.unwrap_or_else(|| {
        let parent = hashfile.parent().unwrap_or(Path::new("."));
        if parent.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            parent.to_path_buf()
        }
    });

    if !args.quiet {
        eprintln!(
            "{} 驗證 {} （演算法: {} 根目錄: {}）",
            style("ℹ").cyan(),
            hashfile.display(),
            hash_type.display_name(),
            root_dir.display()
        );
    }

    let buffer_size = args.buffer.0;

    if let Some(n) = args.threads {
        ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .map_err(|e| OneeError::ArgumentError(format!("線程池設定失敗: {e}")))?;
    }

    let results = verify_hash_file(hashfile, &hash_type, &root_dir, buffer_size);

    let mut total = 0usize;
    let mut matched = 0usize;
    let mut mismatched = 0usize;
    let mut errors = 0usize;

    for result in &results {
        total += 1;
        match result {
            Ok((path, true, _)) => {
                matched += 1;
                if !args.quiet {
                    eprintln!("{} {}", style("✔").green(), path.display());
                }
            }
            Ok((path, false, actual)) => {
                mismatched += 1;
                eprintln!(
                    "{} {}  預期 hash 不匹配 (實際: {actual})",
                    style("✘").red(),
                    path.display()
                );
            }
            Err(e) => {
                errors += 1;
                eprintln!("{} 錯誤: {e}", style("✘").red());
            }
        }
    }

    eprintln!(
        "{} 驗證完成: {total} 個檔案, {matched} 匹配, {mismatched} 不匹配, {errors} 錯誤",
        if mismatched == 0 && errors == 0 {
            style("✔").green()
        } else {
            style("✘").red()
        }
    );

    if mismatched > 0 {
        return Err(OneeError::HashMismatch {
            path: hashfile.clone(),
            expected: format!("{mismatched} 個檔案不匹配"),
            actual: "見上方錯誤列表".into(),
        });
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────
//  json 子命令
// ──────────────────────────────────────────────────────────

fn cmd_json(args: cli::JsonArgs) -> Result<()> {
    let path = &args.path;

    if !path.exists() || !path.is_dir() {
        return Err(OneeError::InvalidPath(format!(
            "路徑不存在或不是目錄: {}",
            path.display()
        )));
    }

    if !args.quiet {
        eprintln!("{} 掃描目錄結構: {}", style("ℹ").cyan(), path.display());
    }

    use std::time::SystemTime;
    use onee_checker::fs::{build_file_node, save_file_node_json};

    let node = build_file_node(path)?;
    let container = FileNodeContainer {
        version: String::from("0.1.0"),
        generation_time: onee_checker::fs::to_unix_timestamp(SystemTime::now()),
        nodes: vec![node],
    };

    let current_dir = env::current_dir().map_err(OneeError::Io)?;
    let default_name = format!(
        "{}.{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        "struct.json"
    );

    match &args.output {
        Some(out) if out.to_string_lossy() == "-" => {
            serde_json::to_writer_pretty(stdout().lock(), &container)
                .map_err(OneeError::Serde)?;
        }
        Some(out) => {
            save_file_node_json(&container, out.as_path())?;
            if !args.quiet {
                eprintln!("{} 已儲存: {}", style("✔").green(), out.display());
            }
        }
        None => {
            let output_path = current_dir.join(&default_name);
            save_file_node_json(&container, &output_path)?;
            if !args.quiet {
                eprintln!("{} 已儲存: {}", style("✔").green(), output_path.display());
            }
        }
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────
//  txt 子命令
// ──────────────────────────────────────────────────────────

fn cmd_txt(args: cli::TxtArgs) -> Result<()> {
    let path = &args.path;

    if !path.exists() || !path.is_dir() {
        return Err(OneeError::InvalidPath(format!(
            "路徑不存在或不是目錄: {}",
            path.display()
        )));
    }

    let size_format = args.size.map(|st| match st {
        SizeType::Binary => SizeFormat::Binary,
        SizeType::Decimal => SizeFormat::Decimal,
        SizeType::Raw => SizeFormat::Raw,
    });

    let option = TreeOption {
        size: size_format,
        last_modified: args.modified,
        created_at: args.created,
    };

    let current_dir = env::current_dir().map_err(OneeError::Io)?;
    let default_name = format!(
        "{}.{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        "tree.txt"
    );

    match &args.output {
        Some(out) if out.to_string_lossy() == "-" => {
            let mut stdout = stdout().lock();
            write_tree(&mut stdout, path, &option)?;
        }
        Some(out) => {
            let file = File::create(out.as_path())?;
            let mut writer = BufWriter::new(file);
            write_tree(&mut writer, path, &option)?;
            if !args.quiet {
                eprintln!("{} 已儲存: {}", style("✔").green(), out.display());
            }
        }
        None => {
            let output_path = current_dir.join(&default_name);
            let file = File::create(&output_path)?;
            let mut writer = BufWriter::new(file);
            write_tree(&mut writer, path, &option)?;
            if !args.quiet {
                eprintln!("{} 已儲存: {}", style("✔").green(), output_path.display());
            }
        }
    }

    Ok(())
}
