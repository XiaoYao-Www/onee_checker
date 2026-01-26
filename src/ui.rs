use crate::fs_utils::FileEntry;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};

/// 彈出視窗類型枚舉
#[derive(PartialEq, Clone, Copy)]
pub enum PopupType {
    Info,
    Warning,
    Error,
}

/// 彈出視窗的狀態
pub struct PopupState {
    pub title: String,
    pub message: String,
    pub popup_type: PopupType,
    pub is_closable: bool,
    pub progress: Option<f64>,
}

/// ### 輔助函式：計算置中區塊
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// ### 繪製彈出視窗 (Popup)
///
/// - popup: 彈出視窗的狀態
pub fn draw_popup(f: &mut Frame, popup: &PopupState) {
    // 根據是否有進度條，調整彈窗大小
    let area = if popup.progress.is_some() {
        centered_rect(60, 35, f.area())
    } else {
        centered_rect(50, 30, f.area())
    };
    
    // 清除該區域背景，避免透視到底下的列表
    f.render_widget(Clear, area);

    // 根據類型設定顏色
    let border_color = match popup.popup_type {
        PopupType::Info => Color::Cyan,
        PopupType::Warning => Color::Yellow,
        PopupType::Error => Color::Red,
    };

    let mut block = Block::default()
        .title(popup.title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(Color::DarkGray));

    if popup.is_closable {
        block = block.title_bottom(ratatui::text::Line::from(" [Enter] 關閉 ").alignment(Alignment::Center));
    }

    // 先繪製外框
    f.render_widget(block, area);

    // 取得內部繪製區塊
    let inner_area = area.inner(ratatui::layout::Margin { vertical: 1, horizontal: 2 });

    // 根據是否有進度條，切分訊息區和進度條區
    let (message_area, progress_area) = if popup.progress.is_some() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner_area);
        (chunks[0], Some(chunks[1]))
    } else {
        (inner_area, None)
    };

    // 繪製訊息
    let paragraph = Paragraph::new(popup.message.as_str())
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));
    f.render_widget(paragraph, message_area);

    // 如果有進度，繪製進度條
    if let (Some(ratio), Some(p_area)) = (popup.progress, progress_area) {
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
            .ratio(ratio.clamp(0.0, 1.0))
            .label(format!("{:.0}%", ratio * 100.0));
        f.render_widget(gauge, p_area);
    }
}

/// ### 顯示檔案列表
///
/// 顯示檔案選擇列表
///
/// - f 輸入Frame
/// - entries 內容列表
/// - selected 選取項目索引
/// - current_path 目前路徑
pub fn draw_file_list(f: &mut Frame, entries: &[FileEntry], selected: usize, current_path: &str) {
    // 預先計算不含進度條時的列表可見高度，以決定是否需要顯示進度條
    // Frame總高度 - 上下margin(2) - 路徑區塊(3) - 預設說明區塊(3) - 列表區塊邊框(2) = 總高度 - 10
    let list_visible_height_without_bar = f.area().height.saturating_sub(10);
    let show_progress_bar = entries.len() as u16 > list_visible_height_without_bar;

    // 根據是否顯示進度條，動態決定底部區塊的高度
    let bottom_size = if show_progress_bar { 4 } else { 3 };

    // 垂直切分主要區塊
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),           // 上面：當前路徑
            Constraint::Min(0),              // 中間：檔案列表
            Constraint::Length(bottom_size), // 下面：進度條與功能說明
        ])
        .split(f.area());

    // ----------------- 上面：當前路徑 -----------------
    let path_block: Paragraph<'_> = Paragraph::new(current_path)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("當前路徑"),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(path_block, chunks[0]);

    // ----------------- 中間：檔案列表 -----------------
    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, e): (usize, &FileEntry)| {
            let symbol: &str = if e.is_dir { "📁" } else { "📄" };
            let distance = i.abs_diff(selected);
            let indent = " ".repeat(3_usize.saturating_sub(distance));
            let content = format!("{}{} {}", indent, symbol, e.name);
            ListItem::new(content)
        })
        .collect();

    let list: List<'_> = List::new(items)
        .block(Block::default().borders(ratatui::widgets::Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ); // 選中反白

    let mut state: ListState = ListState::default();
    state.select(Some(selected));

    // 計算 offset 讓選中項盡可能在垂直高度的中間
    let list_height = chunks[1].height.saturating_sub(2) as usize;
    if list_height > 0 {
        let max_offset = entries.len().saturating_sub(list_height);
        let centered_offset = selected.saturating_sub(list_height / 2);
        *state.offset_mut() = centered_offset.min(max_offset);
    }

    f.render_stateful_widget(list, chunks[1], &mut state);

    // 建立功能說明區塊
    let illustrate_block = Paragraph::new(
        "Enter: 進入, Backspace: 返回, q: 離開, h: 計算.sha256, j: 輸出.struct.json, t: 輸出.tree.txt",
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .style(Style::default().fg(Color::White));

    // ----------------- 下面：進度條與功能說明 -----------------
    if show_progress_bar {
        // 如果需要顯示進度條，將底部區塊再切分成兩部分
        let bottom_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // 進度條
                Constraint::Length(3), // 功能說明
            ])
            .split(chunks[2]);

        // 渲染進度條
        let total_entries = entries.len();
        let progress_ratio = if total_entries > 0 {
            (selected + 1) as f64 / total_entries as f64
        } else {
            0.0
        };
        let progress_label = format!("{}/{}", selected + 1, total_entries);
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
            .ratio(progress_ratio)
            .label(progress_label);
        f.render_widget(gauge, bottom_chunks[0]);

        // 渲染功能說明
        f.render_widget(illustrate_block, bottom_chunks[1]);
    } else {
        // 如果不需要，則整個底部區塊都用來顯示功能說明
        f.render_widget(illustrate_block, chunks[2]);
    }
}