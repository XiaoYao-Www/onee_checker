// ui.rs

use ratatui::{
    backend::Backend,
    widgets::{Block, Borders, List, ListItem, ListState},
    layout::{Layout, Constraint, Direction},
    text::{Line, Span},
    Frame,
};
use crate::fs_utils::FileEntry;

/// 顯示檔案列表的函式
pub fn draw_file_list(f: &mut Frame, entries: &[FileEntry], selected: usize) {
    // 以單一區塊填滿畫面
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    // 將每個項目包裝成 ListItem
    let items: Vec<ListItem> = entries.iter().map(|e| {
        let symbol = if e.is_dir { "[D]" } else { "   " };
        let content = format!("{} {}", symbol, e.name);
        ListItem::new(Line::from(content))
    }).collect();

    // 建立 List widget，並設定標題與邊框
    let list = List::new(items)
        .block(Block::default().title("檔案總管").borders(Borders::ALL))
        .highlight_symbol(">>");

    // 設定 ListState 來標示選取項目
    let mut state = ListState::default();
    state.select(Some(selected));

    // 將 List 畫到畫面上
    f.render_stateful_widget(list, chunks[0], &mut state);
}
