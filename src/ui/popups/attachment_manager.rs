use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::app::App;
use crate::ui::widgets::centered_rect;

pub fn draw_attachment_manager(f: &mut Frame, app: &App, main_area: Rect) {
    let area = centered_rect(60, 50, main_area);
    f.render_widget(Clear, area);

    // Create list items
    let items: Vec<ListItem> = app
        .pending_attachments
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let filename = std::path::Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(path);
            
            let content = format!("  {}. {}", i + 1, filename);
            let style = if i == app.attachment_list_state {
                Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.text)
            };
            
            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(" Manage Attachments ({}) ", app.pending_attachments.len());
    
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.success))
            .title(title)
            .title_bottom(" j/k: Navigate | d/Del: Remove | ESC: Close "),
    );

    let mut list_state = ListState::default();
    list_state.select(Some(app.attachment_list_state));
    f.render_stateful_widget(list, area, &mut list_state);
}
