use ratatui::{layout::Rect, style::Color};

/// Get color for status name
pub fn get_status_color(status: &str) -> Color {
    match status.to_lowercase().as_str() {
        s if s.contains("new") => Color::Cyan,
        s if s.contains("progress") => Color::Yellow,
        s if s.contains("feedback") => Color::Magenta,
        s if s.contains("resolved") => Color::Green,
        s if s.contains("closed") => Color::DarkGray,
        _ => Color::White,
    }
}

/// Get color for priority name
pub fn get_priority_color(priority: &str) -> Color {
    match priority.to_lowercase().as_str() {
        s if s.contains("urgent") || s.contains("immediate") => Color::Red,
        s if s.contains("high") => Color::LightRed,
        s if s.contains("normal") => Color::White,
        s if s.contains("low") => Color::DarkGray,
        _ => Color::White,
    }
}

/// Create a centered rectangle for modals/popups
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Get loading animation frame
pub fn get_loading_spinner() -> String {
    let frames = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

    // Use unwrap_or_default() to handle clock issues gracefully
    // If system clock is before Unix epoch, just use 0
    let frame_idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default() // ✅ Safe: returns Duration::ZERO if clock is wrong
        .as_millis()
        / 100) as usize
        % frames.len();

    frames[frame_idx].to_string()
}
