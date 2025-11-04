mod forms;
mod main_screen;
mod popups;
mod status_bar;
mod widgets;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::{App, InputMode, Screen};

pub use forms::{draw_bulk_edit_form, draw_create_issue_form, draw_reply_form};
pub use main_screen::draw_main_screen;
pub use popups::{
    draw_attachment_manager, draw_config, draw_error_popup, draw_file_explorer, draw_help, draw_image_viewer, draw_issue_popup,
};
pub use status_bar::draw_status_bar;

/// Main draw function - routes to appropriate screen
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4)].as_ref())
        .split(f.area());

    match app.screen {
        Screen::Main => {
            draw_main_screen(f, app, chunks[0]);
            // Draw popups on top if needed
            if app.show_issue_popup {
                draw_issue_popup(f, app, chunks[0]);
            }
            if app.show_create_issue_form {
                draw_create_issue_form(f, app, chunks[0]);
            }
            if app.input_mode == InputMode::ReplyingToIssue {
                draw_reply_form(f, app, chunks[0]);
            }
            // Draw bulk edit form
            if app.bulk_edit_form.is_some() {
                draw_bulk_edit_form(f, app, chunks[0]);
            }
            // Draw help popup
            if app.show_help_popup {
                draw_help(f, app, chunks[0]);
            }
            // Draw image viewer on top of everything
            if app.show_image_viewer {
                draw_image_viewer(f, app, chunks[0]);
            }
            // Draw error popup on top of everything
            if app.show_error_popup {
                draw_error_popup(f, app, chunks[0]);
            }
            // Draw attachment manager
            if app.input_mode == InputMode::ManagingAttachments {
                draw_attachment_manager(f, app, chunks[0]);
            }
            // Draw file explorer
            if app.input_mode == InputMode::AddingAttachment && app.file_explorer.is_some() {
                draw_file_explorer(f, app, chunks[0]);
            }
        }
        Screen::Config => draw_config(f, app, chunks[0]),
    }

    draw_status_bar(f, app, chunks[1]);
}
