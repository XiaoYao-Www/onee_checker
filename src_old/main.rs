// 主要程式，負責應用主要邏輯
#![allow(non_snake_case)]

mod fs_utils; // 檔案系統操作相關（列出目錄、JSON/文字樹）
mod hash_utils_old; // Hash 計算相關
mod type_define; // 通用類型定義
mod ui; // UI 顯示相關

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::collections::HashMap;
use std::fs;
use std::io::{Result, Stdout, stdout};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::type_define::HashResult;

fn main() -> Result<()> {
    enable_raw_mode()?; // 進入 raw mode
    execute!(stdout(), EnterAlternateScreen)?; // 進入替代螢幕緩衝

    // 建立終端機介面
    let mut terminal: Terminal<CrosstermBackend<Stdout>> =
        Terminal::new(CrosstermBackend::new(stdout()))?;

    // 創建導航用變數
    let mut current_path: PathBuf = std::env::current_dir()?; // 目前路徑
    let mut selected: usize = 0; // 列表選取索引
    let mut last_selected_map: HashMap<PathBuf, usize> = HashMap::new(); // 上次索引紀錄

    // 應用狀態變數
    let mut popup: Option<ui::PopupState> = None;
    let mut current_task: Option<type_define::Task> = None;

    loop {
        // 處理背景任務，更新 UI 狀態
        if let Some(task) = &current_task {
            // 處理進度更新
            while let Ok(Some(msg)) = task.tryRecvProgress() {
                let Some(popupState) = popup.as_mut() else {
                    continue;
                };

                match msg {
                    type_define::TaskProgress::Hash { total, current } => {
                        // 起始狀態（還沒開始跑）
                        let Some(cur) = current else {
                            popupState.message = format!("找到 {} 個檔案，開始計算...", total);
                            if total == 0 {
                                popupState.progress = Some(1.0);
                            }
                            continue;
                        };

                        // 已完成
                        if cur == total {
                            popupState.message = "計算完成".to_string();
                            popupState.progress = Some(1.0);
                            continue;
                        }

                        // 進行中
                        popupState.message = format!("正在計算[{}/{}]...", cur, total);
                        popupState.progress = Some(cur as f64 / total as f64);
                    }
                }
            }

            // 處理任務結果
            while let Ok(Some(task_result)) = task.tryRecvResult() {
                match task_result {
                    type_define::TaskResult::Hash(result) => {
                        popup = match result {
                            Ok(type_define::HashResult { root_path, hash_type, data }) => {
                                let output_filename = format!( // 組合輸出檔名
                                    "{}.{}",
                                    root_path.file_name().unwrap_or_default().to_string_lossy(),
                                    hash_utils_old::hashSuffix(&hash_type)
                                );

                                let output_path: PathBuf = root_path.with_file_name(output_filename); // 組合輸出路徑

                                match hash_utils_old::saveChecksums(&data, &output_path) { // 設定彈窗
                                    Ok(_) => Some(ui::PopupState {
                                        title: "完成".to_string(),
                                        message: format!(
                                            "校驗檔已儲存至:\n{}",
                                            output_path.display()
                                        ),
                                        popup_type: ui::PopupType::Info,
                                        is_closable: true,
                                        progress: None,
                                    }),
                                    Err(e) => Some(ui::PopupState {
                                        title: "錯誤".to_string(),
                                        message: format!("儲存校驗檔失敗: {}", e),
                                        popup_type: ui::PopupType::Error,
                                        is_closable: true,
                                        progress: None,
                                    }),
                                }
                            },
                            Err(e) => Some(ui::PopupState {
                                title: "錯誤".to_string(),
                                message: format!("計算 Hash 失敗: {}", e),
                                popup_type: ui::PopupType::Error,
                                is_closable: true,
                                progress: None,
                            }),
                        }
                    }
                }
                current_task = None;
            }
        }

        let entries: Vec<fs_utils::FileEntry> = fs_utils::list_dir(&current_path)?; // 列出目錄下的檔案和資料夾

        // 更新介面顯示
        terminal.draw(|f: &mut ratatui::Frame<'_>| {
            ui::draw_file_list(
                f,
                &entries,
                selected,
                current_path.to_string_lossy().as_ref(),
            );

            // 如果有彈出視窗，最後繪製在最上層
            if let Some(p) = &popup {
                ui::draw_popup(f, p);
            }
        })?;

        // 在 250ms 內檢查是否有鍵盤事件
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // 如果彈出視窗開啟，且允許手動關閉，攔截按鍵
                if let Some(p) = &mut popup {
                    if p.is_closable && (key.code == KeyCode::Enter || key.code == KeyCode::Esc) {
                        popup = None;
                    }
                    // 視窗開啟時，忽略其他操作
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => break, // 按 'q' 離開程式
                    KeyCode::Up => {
                        // 向上移動選取
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        // 向下移動選取
                        if selected < entries.len().saturating_sub(1) {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        // 如果選到的是資料夾，按 Enter 進入該資料夾
                        if entries[selected].is_dir {
                            // 存入目前路徑的 last_selected
                            last_selected_map.insert(current_path.clone(), selected);
                            current_path.push(&entries[selected].name);

                            // 取新資料夾的上次選取，如果沒有就 0
                            selected = *last_selected_map.get(&current_path).unwrap_or(&0);
                        }
                    }
                    KeyCode::Backspace => {
                        // 按 Backspace 返回上層資料夾
                        if current_path.pop() {
                            // 回上一層資料夾時使用上一次選取
                            selected = *last_selected_map.get(&current_path).unwrap_or(&0);

                            // 避免超出目前資料夾長度
                            let entries_len = fs_utils::list_dir(&current_path)?.len();
                            if selected >= entries_len {
                                selected = entries_len.saturating_sub(1);
                            }
                        }
                    }
                    KeyCode::Char('h') => {
                        // 按 'h'：計算選取檔案/資料夾的 SHA256
                        if current_task.is_some() {
                            continue;
                        } // 避免重複啟動任務
                        hashing_total_files = None;

                        let (progress_tx, progress_rx) = mpsc::channel();
                        let (result_tx, result_rx) = mpsc::channel();
                        let entry_path = current_path.join(&entries[selected].name);

                        // 1. 啟動背景執行緒
                        thread::spawn(move || {
                            let result = hash_utils_old::hash_selected(&entry_path, progress_tx);
                            // 計算完成後，將結果發送回主執行緒
                            let _ =
                                result_tx.send(TaskResult::Hash(result.map(|r| (r.1, entry_path))));
                        });

                        // 2. 設定當前任務狀態
                        current_task = Some(Task {
                            progress_rx,
                            result_rx,
                        });

                        // 3. 立即顯示「處理中」彈窗
                        popup = Some(PopupState {
                            title: "處理中".to_string(),
                            message: "準備計算 SHA256...".to_string(),
                            popup_type: PopupType::Info,
                            is_closable: false,
                            progress: Some(0.0),
                        });
                    }
                    KeyCode::Char('j') => {
                        // 按 'j'：輸出選取資料夾的 JSON 結構
                        let entry: &fs_utils::FileEntry = &entries[selected];
                        if entry.is_dir {
                            let path: PathBuf = current_path.join(&entry.name);
                            match fs_utils::get_json_string(&path) {
                                Ok(json_str) => {
                                    let save_path =
                                        path.with_file_name(format!("{}.struct.json", entry.name));
                                    match fs::write(&save_path, &json_str) {
                                        Ok(_) => {
                                            popup = Some(PopupState {
                                                title: "完成".to_string(),
                                                message: format!(
                                                    "JSON 結構已輸出至 {}",
                                                    save_path
                                                        .file_name()
                                                        .unwrap()
                                                        .to_string_lossy()
                                                ),
                                                popup_type: PopupType::Info,
                                                is_closable: true,
                                                progress: None,
                                            });
                                        }
                                        Err(e) => {
                                            popup = Some(PopupState {
                                                title: "錯誤".to_string(),
                                                message: format!("儲存檔案失敗: {}", e),
                                                popup_type: PopupType::Error,
                                                is_closable: true,
                                                progress: None,
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    popup = Some(PopupState {
                                        title: "錯誤".to_string(),
                                        message: format!("生成 JSON 失敗: {}", e),
                                        popup_type: PopupType::Error,
                                        is_closable: true,
                                        progress: None,
                                    });
                                }
                            }
                        } else {
                            // 錯誤提示
                            popup = Some(PopupState {
                                title: "錯誤".to_string(),
                                message: "僅支援資料夾輸出 JSON".to_string(),
                                popup_type: PopupType::Error,
                                is_closable: true,
                                progress: None,
                            });
                        }
                    }
                    KeyCode::Char('t') => {
                        // 按 't'：輸出選取資料夾的文字樹結構
                        let entry: &fs_utils::FileEntry = &entries[selected];
                        if entry.is_dir {
                            let path: PathBuf = current_path.join(&entry.name);
                            match fs_utils::get_tree_string(&path) {
                                Ok(tree_str) => {
                                    let save_path =
                                        path.with_file_name(format!("{}.tree.txt", entry.name));
                                    match fs::write(&save_path, &tree_str) {
                                        Ok(_) => {
                                            popup = Some(PopupState {
                                                title: "完成".to_string(),
                                                message: format!(
                                                    "文字樹已輸出至 {}",
                                                    save_path
                                                        .file_name()
                                                        .unwrap()
                                                        .to_string_lossy()
                                                ),
                                                popup_type: PopupType::Info,
                                                is_closable: true,
                                                progress: None,
                                            });
                                        }
                                        Err(e) => {
                                            popup = Some(PopupState {
                                                title: "錯誤".to_string(),
                                                message: format!("儲存檔案失敗: {}", e),
                                                popup_type: PopupType::Error,
                                                is_closable: true,
                                                progress: None,
                                            });
                                        }
                                    }
                                }
                                Err(e) => {
                                    popup = Some(PopupState {
                                        title: "錯誤".to_string(),
                                        message: format!("生成文字樹失敗: {}", e),
                                        popup_type: PopupType::Error,
                                        is_closable: true,
                                        progress: None,
                                    });
                                }
                            }
                        } else {
                            popup = Some(PopupState {
                                title: "錯誤".to_string(),
                                message: "僅支援資料夾輸出文字樹".to_string(),
                                popup_type: PopupType::Error,
                                is_closable: true,
                                progress: None,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 離開時還原終端機狀態
    disable_raw_mode()?; // 關閉 raw mode
    execute!(stdout(), LeaveAlternateScreen)?; // 離開替代螢幕，還原原本螢幕內容
    Ok(())
}
