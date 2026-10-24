// Copyright (c) 2026 逍遙 (XiaoYao). Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Onee Checker CLI — 薄調度層，業務邏輯全在 library。
//!
//! 設計原則：
//! - stdout → 機器可解析的資料（hash 檔內容、JSON、純文字樹）
//! - stderr → 人類可讀的訊息（進度、錯誤、狀態）
//! - exit code → 0=成功 1=hash不匹配 2=執行/I/O錯誤 3=參數或格式錯誤 4=無可處理項目

use std::fs::File;
use std::io::{stdout, BufWriter, Write};
use std::path::{Path, PathBuf};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::{ThreadPool, ThreadPoolBuilder};

use onee_checker::algorithm::{HashAlgo, HashData, HashMode, HashType};
use onee_checker::cli::{self, Cli, Commands, SizeType};
use onee_checker::error::OneeError;
use onee_checker::error::Result;
use onee_checker::fs::{
    build_file_node, list_files, save_file_node_json, save_hash_file, validate_path,
    FileNodeContainer,
};
use onee_checker::hash::{
    compute_hashes_parallel, compute_hashes_parallel_with_pool, compute_multi_hashes_parallel,
    compute_multi_hashes_parallel_with_pool,
};
use onee_checker::hash::{verify_hash_file, verify_hash_file_with_pool};
use onee_checker::report::{EntryReport, EntryStatus, VerifyReport};
use onee_checker::tree::{write_tree, SizeFormat, TreeOption};
use std::time::SystemTime;

fn main() {
    let cli = match <Cli as clap::Parser>::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = match error.kind() {
                clap::error::ErrorKind::DisplayHelp
                | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                | clap::error::ErrorKind::DisplayVersion => 0,
                _ => 3,
            };
            let _ = error.print();
            std::process::exit(exit_code);
        }
    };

    if cli.no_color {
        console::set_colors_enabled(false);
    }

    let result = match cli.command {
        Commands::Hash(args) => cmd_hash(args, cli.format).map(|()| 0),
        Commands::Verify(args) => cmd_verify(args, cli.format),
        Commands::Json(args) => cmd_json(args).map(|()| 0),
        Commands::Txt(args) => cmd_txt(args).map(|()| 0),
        Commands::Check(args) => cmd_check(args, cli.format),
        Commands::Diff(args) => cmd_diff(args, cli.format),
    };

    match result {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{} {}", style("✘").red(), err);
            std::process::exit(err.exit_code());
        }
    }
}

// ──────────────────────────────────────────────────────────
//  hash 子命令
// ──────────────────────────────────────────────────────────

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn cmd_hash(args: cli::HashArgs, format: onee_checker::report::OutputFormat) -> Result<()> {
    if args.digest_only
        && args.output.is_some()
        && args.output.as_ref().unwrap().to_string_lossy() != "-"
    {
        return Err(OneeError::ArgumentError("指定 --digest-only 時 --output 必須為 '-'".into()));
    }

    let path = &args.path;
    validate_path(path).map_err(OneeError::InvalidPath)?;

    let algos: Vec<(HashAlgo, HashType)> = if args.algo.is_empty() {
        vec![(HashAlgo::Sha256, HashType::SHA256)]
    } else {
        if args.length.is_some() {
            for algo in &args.algo {
                if !algo.can_specify_length() {
                    return Err(OneeError::UnsupportedLength { algorithm: format!("{algo:?}") });
                }
            }
        }
        args.algo
            .iter()
            .map(|a: &HashAlgo| a.to_hash_type(args.length).map(|ht: HashType| (a.clone(), ht)))
            .collect::<Result<Vec<(HashAlgo, HashType)>>>()?
    };

    let pool: Option<ThreadPool> = args
        .threads
        .map(|n| {
            ThreadPoolBuilder::new()
                .num_threads(n)
                .build()
                .map_err(|e| OneeError::ArgumentError(format!("線程池建立失敗: {e}")))
        })
        .transpose()?;

    let files = list_files(path)?;
    if files.is_empty() {
        return Err(OneeError::NothingToDo);
    }

    let buffer_size = args.buffer.0;

    if algos.len() > 1 {
        let all_hash_types: Vec<HashType> = algos.iter().map(|(_, ht)| ht.clone()).collect();
        let pb: Option<ProgressBar> = if args.quiet {
            None
        } else {
            let bar = ProgressBar::new(files.len() as u64);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .expect("Progress bar template 格式正確")
                    .progress_chars("#>-"),
            );
            Some(bar)
        };

        let all_results = match &pool {
            Some(p) => {
                compute_multi_hashes_parallel_with_pool(p, &files, &all_hash_types, buffer_size)
            }
            None => compute_multi_hashes_parallel(&files, &all_hash_types, buffer_size),
        };

        let mut first_error = None;
        for (i, (_, hash_type)) in algos.iter().enumerate() {
            let mut hash_data: Vec<HashData> = Vec::with_capacity(files.len());
            for result in &all_results[i] {
                match result {
                    Ok(data) => hash_data.push(data.clone()),
                    Err(e) => {
                        if first_error.is_none() {
                            first_error = Some(OneeError::Io(std::io::Error::other(e.to_string())));
                        }
                        if args.strict {
                            break;
                        }
                        if i == 0 {
                            eprintln!("{} 錯誤: {e}", style("✘").red());
                        }
                    }
                }
                if i == 0 {
                    if let Some(ref bar) = pb {
                        bar.inc(1);
                    }
                }
            }

            if args.strict {
                if let Some(err) = first_error.take() {
                    return Err(err);
                }
            }

            hash_data.sort_by(|a, b| a.path.cmp(&b.path));
            if args.digest_only || first_error.is_none() {
                save_hash_results(
                    &hash_data,
                    hash_type,
                    path,
                    args.output.as_ref(),
                    &OutputOptions {
                        format,
                        digest_only: args.digest_only,
                        multi_algo: true,
                        mode: args.mode(),
                    },
                )?;
            }
        }
        if let Some(bar) = pb {
            bar.finish_and_clear();
        }
        return match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        };
    }

    let (_, hash_type) = &algos[0];
    let pb: Option<ProgressBar> = if args.quiet {
        None
    } else {
        let bar = ProgressBar::new(files.len() as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .expect("Progress bar template 格式正確")
                .progress_chars("#>-"),
        );
        Some(bar)
    };

    let results = match &pool {
        Some(p) => compute_hashes_parallel_with_pool(p, &files, hash_type, buffer_size),
        None => compute_hashes_parallel(&files, hash_type, buffer_size),
    };

    let mut hash_data: Vec<HashData> = Vec::with_capacity(results.len());
    let mut first_error = None;
    for result in results {
        match result {
            Ok(data) => hash_data.push(data),
            Err(e) => {
                eprintln!("{} 錯誤: {e}", style("✘").red());
                if first_error.is_none() {
                    first_error = Some(e);
                }
                if args.strict {
                    break;
                }
            }
        }
        if let Some(ref bar) = pb {
            bar.inc(1);
        }
    }
    if let Some(bar) = pb {
        bar.finish_and_clear();
    }
    if args.strict {
        if let Some(err) = first_error.take() {
            return Err(err);
        }
    }

    hash_data.sort_by(|a, b| a.path.cmp(&b.path));
    save_hash_results(
        &hash_data,
        hash_type,
        path,
        args.output.as_ref(),
        &OutputOptions {
            format,
            digest_only: args.digest_only,
            multi_algo: false,
            mode: args.mode(),
        },
    )?;
    Ok(())
}

/// `hash` 子命令的輸出選項（避免 `save_hash_results` 參數過多）。
struct OutputOptions {
    format: onee_checker::report::OutputFormat,
    digest_only: bool,
    multi_algo: bool,
    mode: HashMode,
}

fn save_hash_results(
    hash_data: &[HashData],
    hash_type: &HashType,
    path: &Path,
    output: Option<&PathBuf>,
    options: &OutputOptions,
) -> Result<()> {
    let root = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };

    // `--digest-only`：只輸出 hex 值（多演算法時加上演算法前綴）
    if options.digest_only {
        let mut stdout = BufWriter::new(stdout().lock());
        for entry in hash_data {
            if options.multi_algo {
                writeln!(stdout, "{}: {}", hash_type.display_name(), entry.hash_hex())
                    .map_err(OneeError::Io)?;
            } else {
                writeln!(stdout, "{}", entry.hash_hex()).map_err(OneeError::Io)?;
            }
        }
        stdout.flush().map_err(OneeError::Io)?;
        return Ok(());
    }

    match output {
        Some(out) if out.to_string_lossy() == "-" => {
            let mut stdout = BufWriter::new(stdout().lock());
            for entry in hash_data {
                let rel_path = entry.path.strip_prefix(root).unwrap_or(&entry.path);
                let path_str = rel_path.to_string_lossy().replace('\\', "/");
                onee_checker::report::write_hash_line(
                    &mut stdout,
                    options.format,
                    hash_type.display_name(),
                    &entry.hash_hex(),
                    &path_str,
                    options.mode,
                )
                .map_err(OneeError::Io)?;
            }
            stdout.flush().map_err(OneeError::Io)?;
        }
        Some(out) => {
            save_hash_file(hash_data, out.as_path(), root, options.mode)?;
            eprintln!("{} 已儲存: {}", style("✔").green(), out.display());
        }
        None => {
            let output_path = root.join(format!(
                "{}.{}",
                path.file_name().unwrap_or_default().to_string_lossy(),
                hash_type.suffix()
            ));
            save_hash_file(hash_data, &output_path, root, options.mode)?;
            eprintln!("{} 已儲存: {}", style("✔").green(), output_path.display());
        }
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────
//  verify 子命令
// ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)]
fn cmd_verify(args: cli::VerifyArgs, format: onee_checker::report::OutputFormat) -> Result<i32> {
    let hashfile = &args.hashfile;
    let mode = args.mode();
    if !hashfile.exists() || !hashfile.is_file() {
        return Err(OneeError::InvalidPath(format!("Hash 檔不存在: {}", hashfile.display())));
    }

    let hash_type = if let Some(algo) = &args.algo {
        algo.to_hash_type(None)?
    } else {
        let ext =
            hashfile.extension().map(|e| e.to_string_lossy().to_lowercase()).ok_or_else(|| {
                OneeError::ArgumentError("無法從副檔名推斷演算法，請使用 --algo 手動指定".into())
            })?;
        HashAlgo::from_suffix(&ext)
            .ok_or_else(|| {
                OneeError::UnsupportedAlgorithm(format!("無法從副檔名 .{ext} 推斷演算法"))
            })?
            .to_hash_type(None)?
    };

    let root_dir: PathBuf = args.root.unwrap_or_else(|| match hashfile.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    });
    // 統一為絕對路徑，確保 entry 路徑可正確轉成相對路徑（否則會出現 \\?\ 絕對路徑）
    let canonical_root = root_dir.canonicalize().unwrap_or_else(|_| root_dir.clone());

    // 先嚴格驗證 manifest：格式錯誤或無有效項目屬於輸入錯誤（exit 3）
    let content = std::fs::read_to_string(hashfile).map_err(OneeError::Io)?;
    let parsed_entries = onee_checker::fs::parse_hash_file_strict(&content)?;
    if parsed_entries.is_empty() {
        return Err(OneeError::HashFileParseError {
            line: 0,
            detail: "manifest 沒有有效項目".into(),
        });
    }
    let expected_map: std::collections::HashMap<String, String> = parsed_entries
        .iter()
        .map(|entry| (entry.rel_path.replace('\\', "/"), entry.hash_hex.to_lowercase()))
        .collect();

    if !args.quiet {
        eprintln!(
            "{} 驗證 {} （演算法: {} 根目錄: {}）",
            style("ℹ").cyan(),
            hashfile.display(),
            hash_type.display_name(),
            root_dir.display()
        );
    }

    let results = match args
        .threads
        .map(|n| {
            ThreadPoolBuilder::new()
                .num_threads(n)
                .build()
                .map_err(|e| OneeError::ArgumentError(format!("線程池建立失敗: {e}")))
        })
        .transpose()?
    {
        Some(p) => verify_hash_file_with_pool(&p, hashfile, &hash_type, &root_dir, args.buffer.0),
        None => verify_hash_file(hashfile, &hash_type, &root_dir, args.buffer.0),
    };

    let mut entries = Vec::new();
    for result in results {
        match result {
            Ok((path, matched, actual)) => {
                let status = if matched { EntryStatus::Match } else { EntryStatus::Mismatch };
                let rel = path
                    .strip_prefix(&canonical_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                let mut entry = EntryReport::new(rel.clone(), status);
                entry.expected = expected_map.get(&rel).cloned();
                entry.actual = Some(actual);
                entries.push(entry);
            }
            Err(e) => {
                let mut entry = EntryReport::new(e.to_string(), EntryStatus::Error);
                entry.error = Some(e.to_string());
                entries.push(entry);
                if args.strict {
                    return Err(OneeError::Io(std::io::Error::other(e.to_string())));
                }
            }
        }
    }

    if entries.is_empty() {
        return Err(OneeError::HashFileParseError {
            line: 0,
            detail: "manifest 沒有有效項目".into(),
        });
    }

    let mut report = VerifyReport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        command: "verify".into(),
        hashfile: hashfile.to_string_lossy().into_owned(),
        algorithm: hash_type.display_name().into(),
        root: canonical_root.to_string_lossy().trim_start_matches("\\\\?\\").to_string(),
        summary: onee_checker::report::VerifySummary::from_entries(&entries),
        exit_code: 0,
        entries,
    };
    report.exit_code = report.compute_exit_code();

    if format == onee_checker::report::OutputFormat::Human {
        for entry in &report.entries {
            if !args.quiet {
                match entry.status {
                    EntryStatus::Match => eprintln!("{} {}", style("✔").green(), entry.path),
                    EntryStatus::Mismatch => eprintln!(
                        "{} {} 預期 hash 不匹配 (實際: {})",
                        style("✘").red(),
                        entry.path,
                        entry.actual.as_deref().unwrap_or("?")
                    ),
                    EntryStatus::Error => eprintln!(
                        "{} 錯誤: {}",
                        style("✘").red(),
                        entry.error.as_deref().unwrap_or("未知")
                    ),
                    EntryStatus::Missing => {
                        if !args.ignore_missing {
                            eprintln!("{} 缺失: {}", style("✘").red(), entry.path);
                        }
                    }
                }
            }
        }
        eprintln!(
            "{} 驗證完成: {} 個檔案, {} 匹配, {} 不匹配, {} 錯誤",
            if report.exit_code == 0 { style("✔").green() } else { style("✘").red() },
            report.summary.total,
            report.summary.matched,
            report.summary.mismatched,
            report.summary.errors
        );
    } else {
        onee_checker::report::write_verify_report(&mut stdout().lock(), format, &report, mode)
            .map_err(OneeError::Io)?;
    }

    if let Some(out) = args.report {
        let file = File::create(out).map_err(OneeError::Io)?;
        serde_json::to_writer_pretty(file, &report).map_err(OneeError::Serde)?;
    }

    Ok(report.exit_code)
}

// ──────────────────────────────────────────────────────────
//  json 子命令
// ──────────────────────────────────────────────────────────

#[allow(clippy::needless_pass_by_value)]
fn cmd_json(args: cli::JsonArgs) -> Result<()> {
    let path = &args.path;
    if !path.exists() || !path.is_dir() {
        return Err(OneeError::InvalidPath(format!("路徑不存在或不是目錄: {}", path.display())));
    }
    if !args.quiet {
        eprintln!("{} 掃描目錄結構: {}", style("ℹ").cyan(), path.display());
    }

    let node = build_file_node(path)?;
    let container = FileNodeContainer {
        version: env!("CARGO_PKG_VERSION").to_string(),
        generation_time: onee_checker::fs::to_unix_timestamp(SystemTime::now()),
        nodes: vec![node],
    };

    let current_dir = std::env::current_dir().map_err(OneeError::Io)?;
    let default_name =
        format!("{}.{}", path.file_name().unwrap_or_default().to_string_lossy(), "struct.json");

    match &args.output {
        Some(out) if out.to_string_lossy() == "-" => {
            serde_json::to_writer_pretty(stdout().lock(), &container).map_err(OneeError::Serde)?;
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
//  check 子命令
// ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
fn cmd_check(args: cli::CheckArgs, format: onee_checker::report::OutputFormat) -> Result<i32> {
    let length = match args.length {
        Some(value) => Some(
            u16::try_from(value)
                .map_err(|_| OneeError::UnsupportedLength { algorithm: "check".into() })?,
        ),
        None => None,
    };
    let hash_type = match &args.algo {
        Some(algo) => algo.to_hash_type(length)?,
        None => HashType::SHA256,
    };

    let mut buffer = vec![0u8; 64 * 1024];
    let (target, actual) = if let Some(text) = &args.text {
        let mut hasher = hash_type.create_hasher();
        let digest =
            onee_checker::hash::compute_hash_reader(text.as_bytes(), &mut hasher, &mut buffer)
                .map_err(OneeError::Io)?;
        (format!("(text:{text})"), hex::encode(&digest))
    } else {
        let path = args
            .path
            .as_deref()
            .ok_or_else(|| OneeError::ArgumentError("必須提供檔案路徑或 --text".into()))?;
        if !path.is_file() {
            return Err(OneeError::InvalidPath(format!(
                "路徑不存在或不是檔案: {}",
                path.display()
            )));
        }
        let digest = onee_checker::hash::compute_file_hash(path, &hash_type, &mut buffer)
            .map_err(OneeError::Io)?;
        (path.display().to_string(), hex::encode(&digest))
    };

    let matched = args.expect.iter().any(|expected| {
        let expected = expected.trim();
        if args.ignore_case {
            expected.eq_ignore_ascii_case(&actual)
        } else {
            expected == actual
        }
    });

    let mut entry = EntryReport::new(
        target.clone(),
        if matched { EntryStatus::Match } else { EntryStatus::Mismatch },
    );
    entry.expected = Some(args.expect.join(" "));
    entry.actual = Some(actual.clone());

    match format {
        onee_checker::report::OutputFormat::Json | onee_checker::report::OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(&entry).map_err(OneeError::Serde)?);
        }
        onee_checker::report::OutputFormat::Csv => {
            let mut out = stdout().lock();
            writeln!(out, "path,status,expected,actual").map_err(OneeError::Io)?;
            writeln!(
                out,
                "{},{},{},{}",
                onee_checker::report::csv_escape(&target),
                entry.status.as_str(),
                onee_checker::report::csv_escape(&args.expect.join(" ")),
                actual
            )
            .map_err(OneeError::Io)?;
            out.flush().map_err(OneeError::Io)?;
        }
        _ => {
            if matched {
                println!("OK {target} {} {actual}", hash_type.display_name());
            } else {
                println!(
                    "FAIL {target} {} expected={} actual={actual}",
                    hash_type.display_name(),
                    args.expect.join(",")
                );
            }
            eprintln!(
                "{} 檢查完成: {}（{}）",
                if matched { style("✔").green() } else { style("✘").red() },
                if matched { "匹配" } else { "不匹配" },
                hash_type.display_name()
            );
        }
    }

    Ok(i32::from(!matched))
}

// ──────────────────────────────────────────────────────────
//  diff 子命令
// ──────────────────────────────────────────────────────────

#[allow(clippy::needless_pass_by_value)]
fn cmd_diff(args: cli::DiffArgs, format: onee_checker::report::OutputFormat) -> Result<i32> {
    use std::fs;
    let old_content = fs::read_to_string(&args.old).map_err(OneeError::Io)?;
    let new_content = fs::read_to_string(&args.new).map_err(OneeError::Io)?;
    let (_, old_entries) = onee_checker::manifest::parse_auto(&old_content)?;
    let (_, new_entries) = onee_checker::manifest::parse_auto(&new_content)?;

    let report = onee_checker::manifest::diff(&old_entries, &new_entries);

    match format {
        onee_checker::report::OutputFormat::Json => {
            println!("{}", serde_json::to_string(&report).map_err(OneeError::Serde)?);
        }
        onee_checker::report::OutputFormat::Csv => {
            let mut out = stdout().lock();
            writeln!(out, "kind,path,old_hash,new_hash").map_err(OneeError::Io)?;
            for path in &report.added {
                writeln!(out, "added,{},,", onee_checker::report::csv_escape(path))
                    .map_err(OneeError::Io)?;
            }
            for path in &report.removed {
                writeln!(out, "removed,{},,", onee_checker::report::csv_escape(path))
                    .map_err(OneeError::Io)?;
            }
            for change in &report.changed {
                writeln!(
                    out,
                    "changed,{},{},{}",
                    onee_checker::report::csv_escape(&change.path),
                    change.old_hash,
                    change.new_hash
                )
                .map_err(OneeError::Io)?;
            }
            out.flush().map_err(OneeError::Io)?;
        }
        _ => {
            for path in &report.added {
                println!("+ {path}");
            }
            for path in &report.removed {
                println!("- {path}");
            }
            for change in &report.changed {
                println!("~ {} {} -> {}", change.path, change.old_hash, change.new_hash);
            }
            eprintln!(
                "差異: {} 新增, {} 移除, {} 變更, {} 未變",
                report.added.len(),
                report.removed.len(),
                report.changed.len(),
                report.unchanged
            );
        }
    }

    Ok(i32::from(!report.is_empty()))
}

// ──────────────────────────────────────────────────────────
//  txt 子命令
// ──────────────────────────────────────────────────────────

#[allow(clippy::needless_pass_by_value)]
fn cmd_txt(args: cli::TxtArgs) -> Result<()> {
    let path = &args.path;
    if !path.exists() || !path.is_dir() {
        return Err(OneeError::InvalidPath(format!("路徑不存在或不是目錄: {}", path.display())));
    }

    let size_format = args.size.map(|st| match st {
        SizeType::Binary => SizeFormat::Binary,
        SizeType::Decimal => SizeFormat::Decimal,
        SizeType::Raw => SizeFormat::Raw,
    });

    let option =
        TreeOption { size: size_format, last_modified: args.modified, created_at: args.created };
    let current_dir = std::env::current_dir().map_err(OneeError::Io)?;
    let default_name =
        format!("{}.{}", path.file_name().unwrap_or_default().to_string_lossy(), "tree.txt");

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
