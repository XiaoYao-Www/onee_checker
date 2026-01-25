mod fs_utils; // 檔案系統操作相關（列出目錄、JSON/文字樹）
mod hash_utils;
mod ui; // UI 顯示相關 // Hash 計算相關

use crossterm::event::{Event, KeyCode, KeyEventKind, read};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::collections::HashMap;
use std::fs;
use std::io::{Result, stdout};
use std::path::PathBuf;

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

    loop {
        let entries: Vec<fs_utils::FileEntry> = fs_utils::list_dir(&current_path)?; // 列出目錄下的檔案和資料夾

        // 更新介面顯示
        terminal.draw(|f: &mut ratatui::Frame<'_>| {
            ui::draw_file_list(
                f,
                &entries,
                selected,
                current_path.to_string_lossy().as_ref(),
            );
        })?;

        // 等待並處理一個鍵盤事件
        match read()? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
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
                        let entry: &fs_utils::FileEntry = &entries[selected];
                        let path: PathBuf = current_path.join(&entry.name);
                        let sha_result: (bool, Vec<hash_utils::ShaData>) =
                            hash_utils::hash_selected(&path)?;
                        let save_path: PathBuf =
                            current_path.join(format!("{}.sha256", entry.name));
                        hash_utils::save_checksums(&sha_result.1, &save_path)?;
                    }
                    KeyCode::Char('j') => {
                        // 按 'j'：輸出選取資料夾的 JSON 結構
                        let entry: &fs_utils::FileEntry = &entries[selected];
                        if entry.is_dir {
                            let path: PathBuf = current_path.join(&entry.name);
                            let json_str: String = fs_utils::get_json_string(&path)?;
                            let save_path: PathBuf =
                                current_path.join(format!("{}.struct.json", entry.name));
                            fs::write(&save_path, &json_str)?;
                        }
                    }
                    // KeyCode::Char('t') => {      // 按 't'：輸出選取資料夾的文字樹結構
                    //     let entry: &fs_utils::FileEntry = &entries[selected];
                    //     let path: PathBuf = current_path.join(&entry.name);
                    //     fs_utils::print_txt_tree(&path)?;
                    // },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // 離開時還原終端機狀態
    disable_raw_mode()?; // 關閉 raw mode:contentReference[oaicite:1]{index=1}
    execute!(stdout(), LeaveAlternateScreen)?; // 離開替代螢幕，還原原本螢幕內容
    Ok(())
}
