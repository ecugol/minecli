use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::ui::widgets::centered_rect;

pub fn draw_error_popup(f: &mut Frame, app: &App, main_area: Rect) {
    if let Some(error) = &app.error_message {
        let area = centered_rect(80, 60, main_area);

        // Clear background
        f.render_widget(Clear, area);

        // Split error message into lines for display
        let error_lines: Vec<Line> = error.lines().map(|line| Line::from(line.to_string())).collect();

        let paragraph = Paragraph::new(error_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.error))
                    .title(" Error Details (ESC to close) "),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }
}
