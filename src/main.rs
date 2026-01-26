mod fs_utils; // 檔案系統操作相關（列出目錄、JSON/文字樹）
mod hash_utils; // Hash 計算相關
mod ui; // UI 顯示相關

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::collections::HashMap;
use std::fs;
use std::io::{Result, stdout};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use crate::hash_utils::ProgressMessage;
use crate::ui::{PopupState, PopupType};

/// 用於在執行緒之間傳遞的任務結果
enum TaskResult {
    Hash(Result<(Vec<hash_utils::ShaData>, PathBuf)>),
    // 未來可以擴充 Json(Result<...>), Tree(Result<...>)
}

/// 代表一個正在執行的背景任務
struct Task {
    // 用於接收進度更新
    progress_rx: mpsc::Receiver<ProgressMessage>,
    // 用於接收最終結果
    result_rx: mpsc::Receiver<TaskResult>,
}

fn main() -> Result<()> {
    enable_raw_mode()?; // 進入 raw mode
    execute!(stdout(), EnterAlternateScreen)?; // 進入替代螢幕緩衝

    // 建立終端機介面
    let mut terminal: Terminal<CrosstermBackend<std::io::Stdout>> =
        Terminal::new(CrosstermBackend::new(stdout()))?;

    // 設定起始路徑為目前目錄
    let mut current_path: PathBuf = std::env::current_dir()?;
    let mut selected: usize = 0; // 列表中選取項目索引

    let mut last_selected_map: HashMap<PathBuf, usize> = HashMap::new();
    
    // App 狀態
    let mut popup: Option<PopupState> = None;
    let mut current_task: Option<Task> = None;
    let mut hashing_total_files: Option<usize> = None;

    loop {
        // 處理背景任務，更新 UI 狀態
        if let Some(task) = &current_task {
            // 1. 檢查進度更新 (使用 while let 處理所有待辦訊息)
            while let Ok(msg) = task.progress_rx.try_recv() {
                if let Some(p) = popup.as_mut() {
                    match msg {
                        ProgressMessage::Starting(total) => {
                            hashing_total_files = Some(total);
                            p.message = format!("找到 {} 個檔案，開始計算...", total);
                            if total == 0 {
                                p.progress = Some(1.0);
                            }
                        }
                        ProgressMessage::Hashing(current, name) => {
                            if let Some(total) = hashing_total_files {
                                if total > 0 {
                                    p.progress = Some(current as f64 / total as f64);
                                }
                            }
                            p.message = format!("({}/{}) 正在計算:\n{}", current, hashing_total_files.unwrap_or(0), name);
                        }
                        ProgressMessage::Finished => {
                            p.message = "計算完成，正在整理結果...".to_string();
                            p.progress = Some(1.0);
                        }
                    }
                }
            }

            // 2. 檢查最終結果
            if let Ok(task_result) = task.result_rx.try_recv() {
                match task_result {
                    TaskResult::Hash(result) => {
                        popup = match result {
                            Ok((sha_data, entry_path)) => {
                                let output_filename = format!("{}.sha256", entry_path.file_name().unwrap_or_default().to_string_lossy());
                                let output_path = entry_path.with_file_name(output_filename);
                                match hash_utils::save_checksums(&sha_data, &output_path) {
                                    Ok(_) => Some(PopupState {
                                        title: "完成".to_string(),
                                        message: format!("校驗檔已儲存至:\n{}", output_path.display()),
                                        popup_type: PopupType::Info, is_closable: true, progress: None,
                                    }),
                                    Err(e) => Some(PopupState {
                                        title: "錯誤".to_string(),
                                        message: format!("儲存校驗檔失敗: {}", e),
                                        popup_type: PopupType::Error, is_closable: true, progress: None,
                                    }),
                                }
                            }
                            Err(e) => Some(PopupState {
                                title: "錯誤".to_string(),
                                message: format!("計算 Hash 失敗: {}", e),
                                popup_type: PopupType::Error, is_closable: true, progress: None,
                            }),
                        };
                    }
                }
                current_task = None;
                hashing_total_files = None;
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
                        if current_task.is_some() { continue; } // 避免重複啟動任務
                        hashing_total_files = None;

                        let (progress_tx, progress_rx) = mpsc::channel();
                        let (result_tx, result_rx) = mpsc::channel();
                        let entry_path = current_path.join(&entries[selected].name);
                        
                        // 1. 啟動背景執行緒
                        thread::spawn(move || {
                            let result = hash_utils::hash_selected(&entry_path, progress_tx);
                            // 計算完成後，將結果發送回主執行緒
                            let _ = result_tx.send(TaskResult::Hash(result.map(|r| (r.1, entry_path))));
                        });

                        // 2. 設定當前任務狀態
                        current_task = Some(Task { progress_rx, result_rx });

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
                                    let save_path = path.with_file_name(format!("{}.struct.json", entry.name));
                                    match fs::write(&save_path, &json_str) {
                                        Ok(_) => {
                                            popup = Some(PopupState {
                                                title: "完成".to_string(),
                                                message: format!("JSON 結構已輸出至 {}", save_path.file_name().unwrap().to_string_lossy()),
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
                    },
                    KeyCode::Char('t') => {      // 按 't'：輸出選取資料夾的文字樹結構
                        let entry: &fs_utils::FileEntry = &entries[selected];
                        if entry.is_dir {
                            let path: PathBuf = current_path.join(&entry.name);
                            match fs_utils::get_tree_string(&path) {
                                Ok(tree_str) => {
                                    let save_path = path.with_file_name(format!("{}.tree.txt", entry.name));
                                    match fs::write(&save_path, &tree_str) {
                                        Ok(_) => {
                                            popup = Some(PopupState {
                                                title: "完成".to_string(),
                                                message: format!("文字樹已輸出至 {}", save_path.file_name().unwrap().to_string_lossy()),
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
                    },
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
