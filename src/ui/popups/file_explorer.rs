use ratatui::{
    layout::Rect,
    widgets::Clear,
    Frame,
};

use crate::app::App;
use crate::ui::widgets::centered_rect;

pub fn draw_file_explorer(f: &mut Frame, app: &mut App, main_area: Rect) {
    let area = centered_rect(80, 80, main_area);
    f.render_widget(Clear, area);

    if let Some(explorer) = &mut app.file_explorer {
        f.render_widget_ref(explorer.widget(), area);
    }
}
