// 主程式，負責UI與輸入事件
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{self, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::env;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use console::style;

mod system;
mod types;
mod utils;

// ========== 類型定義 ==========

/// ### 哈希算法
#[derive(Clone, Debug, ValueEnum)]
enum HashAlgo {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Sha3256,
    Sha3512,
    Shake128Xof,
    Shake256Xof,
    Blake2s256,
    Blake2b512,
    Blake3,
}

impl HashAlgo {
    /// ### 取得預設長度(bytes)
    ///
    /// 0 代表該類型無須考慮預設長度
    fn default_length(&self) -> usize {
        match self {
            HashAlgo::Shake128Xof => 32,
            HashAlgo::Shake256Xof => 64,
            HashAlgo::Blake3 => 32,
            _ => 0,
        }
    }

    /// ### 是否允許自訂長度
    fn can_specify_length(&self) -> bool {
        match self {
            HashAlgo::Shake128Xof | HashAlgo::Shake256Xof | HashAlgo::Blake3 => true,
            _ => false,
        }
    }

    // ### 取得對應的 Hasher
    fn get_hasher(&self, len: usize) -> types::Hash::HashType {
        match self {
            HashAlgo::Md5 => types::Hash::HashType::MD5,
            HashAlgo::Sha1 => types::Hash::HashType::SHA1,
            HashAlgo::Sha224 => types::Hash::HashType::SHA224,
            HashAlgo::Sha256 => types::Hash::HashType::SHA256,
            HashAlgo::Sha384 => types::Hash::HashType::SHA384,
            HashAlgo::Sha512 => types::Hash::HashType::SHA512,
            HashAlgo::Sha3256 => types::Hash::HashType::SHA3_256,
            HashAlgo::Sha3512 => types::Hash::HashType::SHA3_512,
            HashAlgo::Shake128Xof => types::Hash::HashType::SHAKE128(len),
            HashAlgo::Shake256Xof => types::Hash::HashType::SHAKE256(len),
            HashAlgo::Blake2s256 => types::Hash::HashType::BLAKE2S256,
            HashAlgo::Blake2b512 => types::Hash::HashType::BLAKE2B512,
            HashAlgo::Blake3 => types::Hash::HashType::BLAKE3(len),
        }
    }
}

// ========== 命令設定 ==========

#[derive(Parser)]
#[command(
    name = "oneechk",
    author = "逍遙 https://github.com/XiaoYao-Www",
    arg_required_else_help = true,
    subcommand_required = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 生成哈希驗證檔
    Hash {
        /// 要較驗的路徑
        #[arg(value_hint = clap::ValueHint::AnyPath)]
        path: PathBuf,

        /// 算法 ( 可多選 )
        #[arg(short, long, value_enum)]
        algo: HashAlgo,

        /// 哈希驗證長度
        #[arg(long, help = "哈希較驗長度(bytes)，未指定的話使用預設值。")]
        length: Option<usize>,

        /// 進度條顯示
        #[arg(short, long, help = "進度條顯示與否")]
        progress: bool,

        /// 線程數
        #[arg(long, help = "允許線程數量")]
        thread: Option<usize>,

        /// buffer 大小
        #[arg(long, help = "算法緩存大小(KB)，預設 1 MB")]
        buffer: Option<usize>,
    },
}

// ========== 主程式 ==========

fn main() -> io::Result<()> {
    let cli: Cli = Cli::parse();

    match cli.command {
        Commands::Hash {
            path,
            algo,
            length,
            progress,
            thread,
            buffer,
        } => {
            // 提取路徑
            let target_path: &Path = Path::new(&path);

            // 驗證路徑
            if let Err(info) = utils::FS::validate_path(target_path) {
                eprintln!("{} 錯誤: {}", style("✘").red(), info);
                std::process::exit(1);
            }

            // 驗證長度輸入設定
            if !algo.can_specify_length() && length.is_some() {
                eprintln!("{} 錯誤: 算法 {:?} 無法指定長度", style("✘").red(), algo);
                std::process::exit(1);
            }

            // 取得對應的 Hasher
            let hash_type: types::hash_types::HashType =
                algo.get_hasher(length.unwrap_or(algo.default_length()));

            // 創建通訊器
            let (tx, rx) = channel();

            // 創建線程池
            let pool: ThreadPool = ThreadPoolBuilder::new()
                .num_threads(thread.unwrap_or(1))
                .build()
                .unwrap();

            // 抓取所有檔案路徑
            let file_paths: Vec<PathBuf> = utils::FS::list_file(target_path)?;

            // 創建進度條
            let progress_bar: Option<ProgressBar> = progress.then(|| {
                let pb = ProgressBar::new(file_paths.len() as u64);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                        .unwrap()
                        .progress_chars("#>-")
                );
                pb
            });

            // 添加任務
            pool.spawn(move || {
                file_paths
                    .par_iter() // 在當前pool創建任務
                    .map_init(
                        || vec![0u8; buffer.unwrap_or(1024) * 1024], // 1 MB
                        |buf: &mut Vec<u8>, path: &PathBuf| {
                            system::Hash::compute_file_hash(path, &hash_type, buf).map(|data| {
                                types::Hash::HashData {
                                    path: path.to_path_buf(),
                                    hash_type: hash_type.clone(),
                                    hash_bytes: data,
                                }
                            })
                        },
                    )
                    .for_each(|result| {
                        tx.send(result).ok();
                    });
            });

            // 回收結果
            let mut hash_result: Vec<types::Hash::HashData> = vec![];

            for result in rx {
                match result {
                    Ok(data) => {
                        hash_result.push(data);
                    }
                    Err(e) => {
                        eprintln!("{} 錯誤: {:?}", style("✘").red(), e);
                    }
                };
                if let Some(bar) = progress_bar.as_ref() {
                    bar.inc(1);
                };
            }

            if let Some(bar) = progress_bar {
                bar.finish_and_clear();
                println!("{} 哈希計算完成", style("✔").green());
            }

            // 寫入檔案
            let current_path: PathBuf = env::current_dir()?;
            let output_path: PathBuf = current_path.join(format!(
                "{}.{}",
                target_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                utils::Hash::hash_suffix(&hash_type)
            ));
            hash_result.sort_by(|a: &types::hash_types::HashData, b: &types::hash_types::HashData| a.path.cmp(&b.path));
            utils::FS::save_hash_to_file(
                &hash_result,
                &output_path,
                target_path.parent().unwrap_or(&current_path),
            )?;

            println!("{} 哈希檔案已儲存在: {}", style("✔").green(), output_path.display());
        }
    };

    Ok(())
}
