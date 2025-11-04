use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use ratatui_image::StatefulImage;

use crate::app::App;
use crate::ui::widgets::centered_rect;

pub fn draw_image_viewer(f: &mut Frame, app: &mut App, main_area: Rect) {
    if let Some(url) = &app.viewing_image_url {
        // Calculate optimal size based on image dimensions
        let (width_percent, height_percent) = if let Some(&(img_width, img_height)) = app.image_dimensions.get(url) {
            // Terminal character cells are roughly 2:1 (height:width) in pixel ratio
            // So we adjust the aspect ratio accordingly
            let screen = main_area;
            let screen_cols = screen.width as f32;
            let screen_rows = screen.height as f32;

            // Convert image pixels to terminal cells (approximate)
            // Assume ~8 pixels per cell width, ~16 pixels per cell height
            let img_cols = (img_width as f32 / 8.0).min(screen_cols);
            let img_rows = (img_height as f32 / 16.0).min(screen_rows);

            // Calculate percentage, with min 30% and max 95%
            let width_pct = (img_cols / screen_cols * 100.0).clamp(30.0, 95.0) as u16;
            let height_pct = (img_rows / screen_rows * 100.0).clamp(30.0, 95.0) as u16;

            (width_pct, height_pct)
        } else {
            // Default to 90% if dimensions not available yet
            (90, 90)
        };

        let area = centered_rect(width_percent, height_percent, main_area);

        // Clear background
        f.render_widget(Clear, area);

        // Find attachment info for title
        let attachment_name = if let Some(issue) = &app.current_issue {
            issue
                .attachments
                .iter()
                .find(|a| {
                    let attachment_url =
                        if a.content_url.starts_with("http://") || a.content_url.starts_with("https://") {
                            a.content_url.clone()
                        } else {
                            format!("{}{}", app.config.redmine_url, a.content_url)
                        };
                    attachment_url == *url
                })
                .map(|a| a.filename.as_str())
                .unwrap_or("Image")
        } else {
            "Image"
        };

        // Check if image is loaded
        if let Some(protocol) = app.attachment_images.get_mut(url) {
            // Create block
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.accent))
                .title(format!(" {} (ESC to close) ", attachment_name));

            f.render_widget(block, area);

            // Render image inside the block
            let inner_area = area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 1,
            });

            // Create StatefulImage widget
            let image = StatefulImage::new();
            f.render_stateful_widget(image, inner_area, protocol);
        } else {
            // Show loading message
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.accent))
                .title(format!(" {} - Loading... ", attachment_name));

            let loading_text = Paragraph::new("Loading image...")
                .block(block)
                .alignment(ratatui::layout::Alignment::Center);

            f.render_widget(loading_text, area);
        }
    }
}
