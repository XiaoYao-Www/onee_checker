mod ui;           // UI 顯示相關
mod fs_utils;     // 檔案系統操作相關（列出目錄、JSON/文字樹）
mod hash_utils;   // Hash 計算相關

use std::io::{stdout, Result};
use std::path::{PathBuf};
use crossterm::event::{Event, KeyCode, KeyEventKind, read};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> Result<()> {
    enable_raw_mode()?;                           // 進入 raw mode
    execute!(stdout(), EnterAlternateScreen)?;    // 進入替代螢幕緩衝

    // 建立終端機介面
    let mut terminal: Terminal<CrosstermBackend<std::io::Stdout>> = Terminal::new(CrosstermBackend::new(stdout()))?;

    // 設定起始路徑為目前目錄
    let mut current_path:PathBuf = std::env::current_dir()?;
    let mut selected: usize = 0; // 列表中選取項目索引

    loop {
        let entries: Vec<fs_utils::FileEntry> = fs_utils::list_dir(&current_path)?; // 列出目錄下的檔案和資料夾

        // 更新介面顯示
        terminal.draw(|f: &mut ratatui::Frame<'_>| {
            ui::draw_file_list(f, &entries, selected, current_path.to_string_lossy().as_ref());
        })?;

        // 等待並處理一個鍵盤事件
        match read()? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press { continue; }
                match key.code {
                    KeyCode::Char('q') => break,  // 按 'q' 離開程式
                    KeyCode::Up => {             // 向上移動選取
                        if selected > 0 {
                            selected -= 1;
                        }
                    },
                    KeyCode::Down => {           // 向下移動選取
                        if selected < entries.len().saturating_sub(1) {
                            selected += 1;
                        }
                    },
                    KeyCode::Enter => {          // 如果選到的是資料夾，按 Enter 進入該資料夾
                        if entries[selected].is_dir {
                            current_path.push(&entries[selected].name);
                            selected = 0;
                        }
                    },
                    KeyCode::Backspace => {      // 按 Backspace 返回上層資料夾
                        if current_path.pop() {
                            selected = 0;
                        }
                    },
                    // KeyCode::Char('h') => {      // 按 'h'：計算選取檔案/資料夾的 SHA256
                    //     let entry: &fs_utils::FileEntry = &entries[selected];
                    //     let path: PathBuf = current_path.join(&entry.name);
                    //     hash_utils::hash_selected(&path)?;
                    // },
                    // KeyCode::Char('j') => {      // 按 'j'：輸出選取資料夾的 JSON 結構
                    //     let entry: &fs_utils::FileEntry = &entries[selected];
                    //     let path: PathBuf = current_path.join(&entry.name);
                    //     fs_utils::print_json_tree(&path)?;
                    // },
                    // KeyCode::Char('t') => {      // 按 't'：輸出選取資料夾的文字樹結構
                    //     let entry: &fs_utils::FileEntry = &entries[selected];
                    //     let path: PathBuf = current_path.join(&entry.name);
                    //     fs_utils::print_txt_tree(&path)?;
                    // },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    // 離開時還原終端機狀態
    disable_raw_mode()?;                           // 關閉 raw mode:contentReference[oaicite:1]{index=1}
    execute!(stdout(), LeaveAlternateScreen)?;     // 離開替代螢幕，還原原本螢幕內容
    Ok(())
}
