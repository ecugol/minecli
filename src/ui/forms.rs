use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use super::widgets::centered_rect;
use crate::app::App;

pub fn draw_reply_form(f: &mut Frame, app: &App, main_area: Rect) {
    use crate::form_field::FieldType;

    let area = centered_rect(70, 70, main_area);
    f.render_widget(Clear, area);

    if let Some(form) = &app.update_issue_form {
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        // Calculate which fields to display based on scroll
        let mut field_rows = Vec::new();
        let mut idx = 0;
        while idx < form.fields.len() {
            // Check if this is status_id and next is priority_id
            if idx + 1 < form.fields.len()
                && form.fields[idx].key == "status_id"
                && form.fields[idx + 1].key == "priority_id"
            {
                // Render both on same row
                field_rows.push((idx, Some(idx + 1)));
                idx += 2;
            } else {
                // Render single field
                field_rows.push((idx, None));
                idx += 1;
            }
        }

        // Calculate constraints for visible rows
        let mut constraints = Vec::new();
        for (field_idx, _maybe_second) in &field_rows {
            let field = &form.fields[*field_idx];
            let height = match field.field_type {
                FieldType::TextArea => 6,
                _ => 3,
            };
            constraints.push(Constraint::Length(height));
        }
        constraints.push(Constraint::Min(2)); // Help text

        let content_area = Rect {
            x: inner_area.x + 1,
            y: inner_area.y + 1,
            width: inner_area.width.saturating_sub(2),
            height: inner_area.height.saturating_sub(2),
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(content_area);

        // Render each row
        for (chunk_idx, (field_idx, maybe_second)) in field_rows.iter().enumerate() {
            if let Some(second_idx) = maybe_second {
                // Render two fields side by side
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[chunk_idx]);

                // Render first field (status)
                render_form_field(app, f, &form.fields[*field_idx], form, *field_idx, row_chunks[0]);
                // Render second field (priority)
                render_form_field(app, f, &form.fields[*second_idx], form, *second_idx, row_chunks[1]);
            } else {
                // Render single field
                render_form_field(app, f, &form.fields[*field_idx], form, *field_idx, chunks[chunk_idx]);
            }
        }

        // Help text (split into two lines)
        let help_idx = field_rows.len();
        
        // Show attachments info and help
        let attachments_count = app.pending_attachments.len();
        
        let help_lines = vec![
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(app.theme.warning)),
                Span::raw(": Next | "),
                Span::styled("Shift+Tab", Style::default().fg(app.theme.warning)),
                Span::raw(": Prev | "),
                Span::styled("@", Style::default().fg(app.theme.warning)),
                Span::raw(": Add file | "),
                Span::styled("#", Style::default().fg(app.theme.warning)),
                Span::raw(": Manage attachments"),
            ]).alignment(Alignment::Center),
            Line::from({
                let mut spans = vec![
                    Span::styled("Ctrl+S", Style::default().fg(app.theme.success)),
                    Span::raw(": Submit | "),
                    Span::styled("ESC", Style::default().fg(app.theme.error)),
                    Span::raw(": Cancel"),
                ];
                if attachments_count > 0 {
                    spans.push(Span::raw(" | "));
                    spans.push(Span::styled(
                        format!("📎 {} attachment(s)", attachments_count),
                        Style::default().fg(app.theme.accent),
                    ));
                }
                spans
            }).alignment(Alignment::Center),
        ];
        
        let help = Paragraph::new(help_lines)
            .style(Style::default().fg(app.theme.text_muted));
        f.render_widget(help, chunks[help_idx]);
    }

    // Outer border
    let title = if app.pending_attachments.is_empty() {
        " Reply to Issue ".to_string()
    } else {
        format!(" Reply to Issue [📎 {}] ", app.pending_attachments.len())
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.success))
        .title(title);
    f.render_widget(block, area);
}

pub fn draw_create_issue_form(f: &mut Frame, app: &App, main_area: Rect) {
    use crate::form_field::FieldType;

    let area = centered_rect(70, 80, main_area);
    f.render_widget(Clear, area);

    if let Some(form) = &app.create_issue_form {
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        // Calculate which fields to display based on scroll
        let mut field_rows = Vec::new();
        let mut idx = 0;
        while idx < form.fields.len() {
            // Check if this is status_id and next is priority_id
            if idx + 1 < form.fields.len()
                && form.fields[idx].key == "status_id"
                && form.fields[idx + 1].key == "priority_id"
            {
                // Render both on same row
                field_rows.push((idx, Some(idx + 1)));
                idx += 2;
            } else {
                // Render single field
                field_rows.push((idx, None));
                idx += 1;
            }
        }

        // Calculate constraints for visible rows
        let mut constraints = Vec::new();
        let mut row_heights = Vec::new();
        for (field_idx, _maybe_second) in &field_rows {
            let field = &form.fields[*field_idx];
            let height = match field.field_type {
                FieldType::TextArea => 6,
                _ => 3,
            };
            row_heights.push(height);
            constraints.push(Constraint::Length(height));
        }
        constraints.push(Constraint::Min(2)); // Help text

        let content_area = Rect {
            x: inner_area.x + 1,
            y: inner_area.y + 1,
            width: inner_area.width.saturating_sub(2),
            height: inner_area.height.saturating_sub(2),
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(content_area);

        // Render each row
        for (chunk_idx, (field_idx, maybe_second)) in field_rows.iter().enumerate() {
            if let Some(second_idx) = maybe_second {
                // Render two fields side by side
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[chunk_idx]);

                // Render first field (status)
                render_form_field(app, f, &form.fields[*field_idx], form, *field_idx, row_chunks[0]);
                // Render second field (priority)
                render_form_field(app, f, &form.fields[*second_idx], form, *second_idx, row_chunks[1]);
            } else {
                // Render single field
                render_form_field(app, f, &form.fields[*field_idx], form, *field_idx, chunks[chunk_idx]);
            }
        }

        // Help text (split into two lines)
        let help_idx = field_rows.len();
        
        // Show attachments info and help
        let attachments_count = app.pending_attachments.len();
        
        let help_lines = vec![
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(app.theme.warning)),
                Span::raw(": Next | "),
                Span::styled("Shift+Tab", Style::default().fg(app.theme.warning)),
                Span::raw(": Prev | "),
                Span::styled("@", Style::default().fg(app.theme.warning)),
                Span::raw(": Add file | "),
                Span::styled("#", Style::default().fg(app.theme.warning)),
                Span::raw(": Manage attachments"),
            ]).alignment(Alignment::Center),
            Line::from({
                let mut spans = vec![
                    Span::styled("Ctrl+S", Style::default().fg(app.theme.success)),
                    Span::raw(": Submit | "),
                    Span::styled("ESC", Style::default().fg(app.theme.error)),
                    Span::raw(": Cancel"),
                ];
                if attachments_count > 0 {
                    spans.push(Span::raw(" | "));
                    spans.push(Span::styled(
                        format!("📎 {} attachment(s)", attachments_count),
                        Style::default().fg(app.theme.accent),
                    ));
                }
                spans
            }).alignment(Alignment::Center),
        ];
        
        let help = Paragraph::new(help_lines)
            .style(Style::default().fg(app.theme.text_muted));
        f.render_widget(help, chunks[help_idx]);
    }

    // Outer border
    let title = if app.pending_attachments.is_empty() {
        " Create New Issue ".to_string()
    } else {
        format!(" Create New Issue [📎 {}] ", app.pending_attachments.len())
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.success))
        .title(title);
    f.render_widget(block, area);
}

fn render_form_field(
    app: &App,
    f: &mut Frame,
    field: &crate::form_field::FormField,
    form: &crate::issue_form::IssueForm,
    field_idx: usize,
    area: Rect,
) {
    use crate::form_field::FieldType;

    let is_focused = field_idx == form.current_field_idx;
    let style = if is_focused {
        Style::default().fg(app.theme.warning)
    } else {
        Style::default()
    };

    // Get field value
    let value = form.get_value(&field.key);

    // Special rendering for SearchableDropdown in search mode - show as a list with search
    if is_focused && field.field_type == FieldType::SearchableDropdown && form.is_search_mode(&field.key) {
        let search_text = form.get_search_text(&field.key);
        let filtered_options = form.get_filtered_options(&field.key);
        let current_id = value.and_then(|v| v.as_option_id());

        // Create list items for filtered options
        let items: Vec<ListItem> = filtered_options
            .iter()
            .map(|opt| {
                let is_selected = current_id == Some(opt.id);
                let content = if is_selected {
                    format!("► {}", opt.name)
                } else {
                    format!("  {}", opt.name)
                };
                let item_style = if is_selected {
                    Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(content).style(item_style)
            })
            .collect();

        // Title with search text and match count
        let required_marker = if field.required { " *" } else { "" };
        let title = if search_text.is_empty() {
            format!(
                "{}{} (█ {} matches)",
                field.label,
                required_marker,
                filtered_options.len()
            )
        } else {
            format!(
                "{}{} ({}█ {} matches)",
                field.label,
                required_marker,
                search_text,
                filtered_options.len()
            )
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.warning))
                .title(title),
        );

        // Calculate selected index for highlighting
        if let Some(current_id) = current_id {
            if let Some(selected_idx) = filtered_options.iter().position(|opt| opt.id == current_id) {
                let mut list_state = ListState::default();
                list_state.select(Some(selected_idx));
                f.render_stateful_widget(list, area, &mut list_state);
                return;
            }
        }

        f.render_widget(list, area);
        return;
    }

    // Render Dropdown and SearchableDropdown as list when focused
    if is_focused && (field.field_type == FieldType::Dropdown || field.field_type == FieldType::SearchableDropdown) {
        let current_id = value.and_then(|v| v.as_option_id());

        // Create list items for all options
        let items: Vec<ListItem> = field
            .options
            .iter()
            .map(|opt| {
                let is_selected = current_id == Some(opt.id);
                let content = if is_selected {
                    format!("► {}", opt.name)
                } else {
                    format!("  {}", opt.name)
                };
                let item_style = if is_selected {
                    Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(content).style(item_style)
            })
            .collect();

        // Title with hint
        let required_marker = if field.required { " *" } else { "" };
        let hint = if field.field_type == FieldType::SearchableDropdown {
            " (j/k or ↑↓ to navigate, / to search)"
        } else {
            " (j/k or ↑↓ to navigate)"
        };
        let title = format!("{}{}{}", field.label, required_marker, hint);

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.warning))
                .title(title),
        );

        // Calculate selected index for highlighting
        if let Some(current_id) = current_id {
            if let Some(selected_idx) = field.options.iter().position(|opt| opt.id == current_id) {
                let mut list_state = ListState::default();
                list_state.select(Some(selected_idx));
                f.render_stateful_widget(list, area, &mut list_state);
                return;
            }
        }

        f.render_widget(list, area);
        return;
    }

    // Normal rendering for unfocused dropdowns and all other field types
    let display_text = match value {
        Some(crate::form_field::FieldValue::Text(s)) => s.clone(),
        Some(crate::form_field::FieldValue::OptionId(Some(id))) => {
            // Find option name by id
            field
                .options
                .iter()
                .find(|opt| opt.id == *id)
                .map(|opt| opt.name.clone())
                .unwrap_or_else(|| format!("ID: {}", id))
        }
        Some(crate::form_field::FieldValue::OptionId(None)) => "(None)".to_string(),
        Some(crate::form_field::FieldValue::Number(Some(n))) => format!("{}%", n),
        Some(crate::form_field::FieldValue::Number(None)) => "0%".to_string(),
        Some(crate::form_field::FieldValue::Float(Some(f))) => format!("{}", f),
        Some(crate::form_field::FieldValue::Float(None)) => String::new(),
        Some(crate::form_field::FieldValue::Boolean(b)) => if *b { "[X]" } else { "[ ]" }.to_string(),
        None => String::new(),
    };

    let title = if field.required {
        format!("{} *", field.label)
    } else {
        field.label.clone()
    };
    
    // Add help text for date fields when focused
    let title_with_help = if is_focused && field.help_text.is_some() {
        format!("{} ({})", title, field.help_text.as_ref().unwrap())
    } else {
        title
    };

    let widget = Paragraph::new(display_text)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title(title_with_help))
        .wrap(Wrap { trim: false });

    f.render_widget(widget, area);
}

pub fn draw_bulk_edit_form(f: &mut Frame, app: &App, main_area: Rect) {
    use crate::form_field::FieldType;

    let area = centered_rect(70, 60, main_area);
    f.render_widget(Clear, area);

    if let Some(form) = &app.bulk_edit_form {
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        // Calculate constraints for fields
        let mut constraints = Vec::new();
        for field in &form.fields {
            let height = match field.field_type {
                FieldType::TextArea => 6,
                _ => 3,
            };
            constraints.push(Constraint::Length(height));
        }
        constraints.push(Constraint::Min(2)); // Help text

        let content_area = Rect {
            x: inner_area.x + 1,
            y: inner_area.y + 1,
            width: inner_area.width.saturating_sub(2),
            height: inner_area.height.saturating_sub(2),
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(content_area);

        // Render each field
        for (i, field) in form.fields.iter().enumerate() {
            render_form_field(app, f, field, form, i, chunks[i]);
        }

        // Help text
        let help_idx = form.fields.len();
        let help = Paragraph::new(format!(
            "Tab: Next field | Shift+Tab: Prev | ↑/↓: Change value | Ctrl+S: Update {} issue(s) | ESC: Cancel",
            app.selected_issues.len()
        ))
        .style(Style::default().fg(app.theme.text_muted))
        .alignment(Alignment::Center);
        f.render_widget(help, chunks[help_idx]);
    }

    // Outer border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.success))
        .title(format!(" Bulk Edit {} Issues ", app.selected_issues.len()));
    f.render_widget(block, area);
}
