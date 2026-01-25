use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    layout::{Layout, Constraint, Direction},
    Frame,
    style::{Style, Color, Modifier}
};
use crate::fs_utils::FileEntry;

/// ### 顯示檔案列表的函式
/// 
/// f 輸入Frame
/// entries 內容列表
/// selected 選取項目索引
pub fn draw_file_list(f: &mut Frame, entries: &[FileEntry], selected: usize, current_path: &str) {
    // 垂直切分成兩塊：上 10% 顯示路徑，下 90% 顯示檔案列表
    let chunks: std::rc::Rc<[ratatui::prelude::Rect]> = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(10),
                      Constraint::Percentage(80),
                      Constraint::Percentage(10)])
        .split(f.area());

    // ----------------- 上面：當前路徑 -----------------
    let path_block: Paragraph<'_> = Paragraph::new(current_path)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("當前路徑"))
        .style(Style::default().fg(Color::White));
    f.render_widget(path_block, chunks[0]);

    // ----------------- 中間：檔案列表 -----------------
    let items: Vec<ListItem> = entries.iter().enumerate().map(|(i, e): (usize, &FileEntry)| {
        let symbol: &str = if e.is_dir { "📁" } else { "  " };
        let content: String = if i == selected {
            format!("{}  {}", symbol, e.name) // 選中行前面多兩個空格
        } else {
            format!("{} {}", symbol, e.name)
        };
        ListItem::new(content)
    }).collect();

    let list: List<'_> = List::new(items)
        .block(Block::default()
            .borders(ratatui::widgets::Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)); // 選中反白

    let mut state: ListState = ListState::default();
    state.select(Some(selected));

    f.render_stateful_widget(list, chunks[1], &mut state);

    // ----------------- 下面：功能說明 -----------------
    let illustrate_block: Paragraph<'_> = Paragraph::new("Enter: 進入, Backspace: 返回, q: 離開")
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .style(Style::default().fg(Color::White));
    f.render_widget(illustrate_block, chunks[2]);
}
