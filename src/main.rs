// 主程式，負責UI與輸入事件
use std::io::{ Result, Stdout, stdout };
use crossterm::execute;
use crossterm::terminal::{ 
    EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode 
};
use ratatui::{ Terminal, backend::CrosstermBackend };

mod types;
mod utils;
mod system;


fn main() -> Result<()>{
    // 開啟原始模式並進入替代螢幕
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    // 建立終端機
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = Terminal::new(CrosstermBackend::new(stdout()))?;

    // 創建系統變數
    let mut command_mode: bool = false; // 是否進入命令模式

    // 推出替代畫面並關閉原始模式
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}