// Re-export all popup drawing functions
mod attachment_manager;
mod error_dialog;
mod file_explorer;
mod image_viewer;
mod journal_helpers;

pub use attachment_manager::draw_attachment_manager;
pub use error_dialog::draw_error_popup;
pub use file_explorer::draw_file_explorer;
pub use image_viewer::draw_image_viewer;

// Keep the larger functions in this file temporarily
// These can be split further in future iterations if needed

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use crate::app::{App, InputMode};
use crate::ui::widgets::centered_rect;

pub fn draw_issue_popup(f: &mut Frame, app: &mut App, main_area: Rect) {
    // This function is large and complex - keeping here for now
    // Can be split further if needed
    // Original implementation from lines 15-403 of old popups.rs
    use journal_helpers::*;

    if let Some(issue) = &app.current_issue {
        let area = centered_rect(80, 80, main_area);

        // Clear the background
        f.render_widget(Clear, area);

        let mut text = Vec::new();

        text.push(Line::from(vec![
            Span::styled(
                "Issue #",
                Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}", issue.id),
                Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
            ),
        ]));
        text.push(Line::from(""));

        text.push(Line::from(vec![
            Span::styled(
                "Subject: ",
                Style::default().fg(app.theme.success).add_modifier(Modifier::BOLD),
            ),
            Span::raw(&issue.subject),
        ]));
        text.push(Line::from(""));

        text.push(Line::from(vec![
            Span::styled("Project: ", Style::default().fg(app.theme.primary)),
            Span::raw(&issue.project.name),
            Span::raw("  "),
            Span::styled("Tracker: ", Style::default().fg(app.theme.primary)),
            Span::raw(&issue.tracker.name),
        ]));

        let status_color = app.theme.get_status_color(&issue.status.name);
        let priority_color = app.theme.get_priority_color(&issue.priority.name);

        text.push(Line::from(vec![
            Span::styled("Status: ", Style::default().fg(app.theme.primary)),
            Span::styled(
                &issue.status.name,
                Style::default().fg(status_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Priority: ", Style::default().fg(app.theme.primary)),
            Span::styled(
                &issue.priority.name,
                Style::default().fg(priority_color).add_modifier(Modifier::BOLD),
            ),
        ]));

        text.push(Line::from(vec![
            Span::styled("Author: ", Style::default().fg(app.theme.primary)),
            Span::raw(&issue.author.name),
        ]));

        if let Some(assigned_to) = &issue.assigned_to {
            text.push(Line::from(vec![
                Span::styled("Assigned: ", Style::default().fg(app.theme.primary)),
                Span::raw(&assigned_to.name),
            ]));
        }

        text.push(Line::from(vec![
            Span::styled("Created: ", Style::default().fg(app.theme.text_muted)),
            Span::raw(issue.created_on.format("%Y-%m-%d %H:%M").to_string()),
            Span::raw("  "),
            Span::styled("Updated: ", Style::default().fg(app.theme.text_muted)),
            Span::raw(issue.updated_on.format("%Y-%m-%d %H:%M").to_string()),
        ]));

        if let Some(done_ratio) = issue.done_ratio {
            let progress_color = if done_ratio >= 80 {
                app.theme.success
            } else if done_ratio >= 50 {
                app.theme.warning
            } else {
                app.theme.error
            };
            text.push(Line::from(vec![
                Span::styled("Progress: ", Style::default().fg(app.theme.primary)),
                Span::styled(
                    format!("{}%", done_ratio),
                    Style::default().fg(progress_color).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        text.push(Line::from(""));
        text.push(Line::from(vec![Span::styled(
            "Description:",
            Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD),
        )]));
        text.push(Line::from("─".repeat(area.width as usize - 4)));

        if let Some(description) = &issue.description {
            // Split by lines and wrap each line properly
            // Use a single Span per line to allow wrapping, but preserve line breaks
            for line in description.lines() {
                // Empty lines should be preserved
                if line.is_empty() {
                    text.push(Line::from(""));
                } else {
                    // Create a single span for the entire line to allow proper wrapping
                    text.push(Line::from(Span::raw(line.to_string())));
                }
            }
        } else {
            text.push(Line::from(Span::styled(
                "No description",
                Style::default().fg(app.theme.text_muted),
            )));
        }

        // Add attachments section
        if !issue.attachments.is_empty() {
            text.push(Line::from(""));
            text.push(Line::from(""));

            // Pagination: show 9 attachments per page
            const ATTACHMENTS_PER_PAGE: usize = 9;
            let total_pages = issue.attachments.len().div_ceil(ATTACHMENTS_PER_PAGE);
            let current_page = app.attachment_page.min(total_pages.saturating_sub(1));
            let start_idx = current_page * ATTACHMENTS_PER_PAGE;
            let end_idx = (start_idx + ATTACHMENTS_PER_PAGE).min(issue.attachments.len());

            text.push(Line::from(vec![Span::styled(
                format!(
                    "Attachments ({}) - Page {}/{}",
                    issue.attachments.len(),
                    current_page + 1,
                    total_pages
                ),
                Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD),
            )]));
            text.push(Line::from("─".repeat(area.width as usize - 4)));

            for (page_idx, attachment) in issue.attachments[start_idx..end_idx].iter().enumerate() {
                let _idx = start_idx + page_idx;
                // Format filesize in human-readable format
                let size_str = if attachment.filesize < 1024 {
                    format!("{} B", attachment.filesize)
                } else if attachment.filesize < 1024 * 1024 {
                    format!("{:.1} KB", attachment.filesize as f64 / 1024.0)
                } else {
                    format!("{:.1} MB", attachment.filesize as f64 / (1024.0 * 1024.0))
                };

                // Check if it's an image
                let is_image = attachment.content_type.as_ref().map_or(false, |ct| ct.starts_with("image/"));
                let icon = if is_image { "🖼️ " } else { "📎 " };

                text.push(Line::from(vec![
                    Span::styled(
                        format!("  {} ", page_idx + 1),
                        Style::default().fg(app.theme.text_muted),
                    ),
                    Span::styled(icon, Style::default().fg(app.theme.primary)),
                    Span::styled(
                        &attachment.filename,
                        Style::default().fg(app.theme.success).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ({}) ", size_str)),
                    Span::styled(
                        format!("by {}", attachment.author.name),
                        Style::default().fg(app.theme.text_muted),
                    ),
                ]));

                if !attachment.description.is_empty() {
                    text.push(Line::from(vec![
                        Span::raw("     "),
                        Span::styled(
                            &attachment.description,
                            Style::default()
                                .fg(app.theme.text_secondary)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]));
                }
            }

            text.push(Line::from(""));
            let mut help_spans = vec![
                Span::styled("  Press ", Style::default().fg(app.theme.text_muted)),
                Span::styled(
                    "1-9",
                    Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to view/open", Style::default().fg(app.theme.text_muted)),
            ];

            if total_pages > 1 {
                help_spans.extend(vec![
                    Span::styled(" | ", Style::default().fg(app.theme.text_muted)),
                    Span::styled("[", Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)),
                    Span::styled(" prev page", Style::default().fg(app.theme.text_muted)),
                    Span::styled(" | ", Style::default().fg(app.theme.text_muted)),
                    Span::styled("]", Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)),
                    Span::styled(" next page", Style::default().fg(app.theme.text_muted)),
                ]);
            }

            text.push(Line::from(help_spans));
        }

        // Add notes/journals section
        if !issue.journals.is_empty() {
            text.push(Line::from(""));
            text.push(Line::from(""));
            text.push(Line::from(vec![Span::styled(
                format!("Notes & Comments ({})", issue.journals.len()),
                Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD),
            )]));
            text.push(Line::from("═".repeat(area.width as usize - 4)));

            for journal in issue.journals.iter() {
                // Show journals that have notes OR details (changes)
                let has_notes = journal.notes.as_ref().map_or(false, |n| !n.trim().is_empty());
                let has_details = !journal.details.is_empty();
                
                if has_notes || has_details {
                    text.push(Line::from(""));

                    // Header with user and date
                    text.push(Line::from(vec![
                        Span::styled("┌─ ", Style::default().fg(app.theme.text_muted)),
                        Span::styled(
                            &journal.user.name,
                            Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(" • ", Style::default().fg(app.theme.text_muted)),
                        Span::styled(
                            journal.created_on.format("%Y-%m-%d %H:%M").to_string(),
                            Style::default().fg(app.theme.text_muted),
                        ),
                    ]));

                    // Note content (if any)
                    if let Some(notes) = &journal.notes {
                        if !notes.trim().is_empty() {
                            // Calculate max width for text (accounting for borders and prefix)
                            // area.width - 4 (borders) - 2 ("│ " prefix)
                            let max_text_width = (area.width as usize).saturating_sub(6);
                            
                            for line in notes.lines() {
                                if line.is_empty() {
                                    text.push(Line::from(vec![
                                        Span::styled("│", Style::default().fg(app.theme.text_muted)),
                                    ]));
                                } else {
                                    // Manually wrap long lines to prevent bleeding
                                    // Use char indices to properly handle UTF-8
                                    let mut remaining = line;
                                    while !remaining.is_empty() {
                                        let chars: Vec<char> = remaining.chars().collect();
                                        
                                        if chars.len() <= max_text_width {
                                            // Fits on one line
                                            text.push(Line::from(vec![
                                                Span::styled("│ ", Style::default().fg(app.theme.text_muted)),
                                                Span::raw(remaining.to_string()),
                                            ]));
                                            break;
                                        } else {
                                            // Need to wrap - find a good break point
                                            let mut break_point = max_text_width;
                                            
                                            // Try to break at a space (search backwards from max_text_width)
                                            for i in (0..max_text_width.min(chars.len())).rev() {
                                                if chars[i] == ' ' {
                                                    break_point = i;
                                                    break;
                                                }
                                            }
                                            
                                            // Convert char index to byte index for split_at
                                            let byte_index = remaining.char_indices()
                                                .nth(break_point)
                                                .map(|(i, _)| i)
                                                .unwrap_or(remaining.len());
                                            
                                            let (current, rest) = remaining.split_at(byte_index);
                                            text.push(Line::from(vec![
                                                Span::styled("│ ", Style::default().fg(app.theme.text_muted)),
                                                Span::raw(current.to_string()),
                                            ]));
                                            remaining = rest.trim_start();
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Show details if there are any changes
                        if !journal.details.is_empty() {
                            text.push(Line::from(vec![Span::styled(
                                "│ ",
                                Style::default().fg(app.theme.text_muted),
                            )]));
                            for detail in &journal.details {
                                // Get human-readable field name and values
                                let (field_display, old_verbose, new_verbose) = format_journal_detail(app, detail);

                                // Get color for the field (before consuming the values)
                                let value_color = get_field_color(app, &detail.name, new_verbose.as_deref());

                                // Build the change line with styled spans
                                let mut spans = vec![
                                    Span::styled("│ ", Style::default().fg(app.theme.text_muted)),
                                    Span::styled("  ", Style::default()),
                                    Span::styled(
                                        field_display,
                                        Style::default()
                                            .fg(app.theme.text_secondary)
                                            .add_modifier(Modifier::BOLD),
                                    ),
                                ];

                                match (old_verbose.as_ref(), new_verbose.as_ref()) {
                                    (Some(old), Some(new)) => {
                                        spans.push(Span::styled(
                                            " changed from ",
                                            Style::default().fg(app.theme.text_muted),
                                        ));
                                        spans.push(Span::styled(
                                            old.clone(),
                                            Style::default()
                                                .fg(app.theme.text_secondary)
                                                .add_modifier(Modifier::ITALIC),
                                        ));
                                        spans.push(Span::styled(" to ", Style::default().fg(app.theme.text_muted)));
                                        spans.push(Span::styled(
                                            new.clone(),
                                            Style::default().fg(value_color).add_modifier(Modifier::BOLD),
                                        ));
                                    }
                                    (None, Some(new)) => {
                                        // Check if this is an attachment addition
                                        if detail.property == "attachment" {
                                            spans.push(Span::styled(
                                                " added",
                                                Style::default().fg(app.theme.text_muted),
                                            ));
                                        } else {
                                            spans.push(Span::styled(
                                                " set to ",
                                                Style::default().fg(app.theme.text_muted),
                                            ));
                                            spans.push(Span::styled(
                                                new.clone(),
                                                Style::default().fg(value_color).add_modifier(Modifier::BOLD),
                                            ));
                                        }
                                    }
                                    (Some(old), None) => {
                                        // Check if this is an attachment deletion
                                        if detail.property == "attachment" {
                                            spans.push(Span::styled(
                                                " deleted",
                                                Style::default().fg(app.theme.text_muted),
                                            ));
                                        } else {
                                            spans.push(Span::styled(
                                                " cleared from ",
                                                Style::default().fg(app.theme.text_muted),
                                            ));
                                            spans.push(Span::styled(
                                                old.clone(),
                                                Style::default()
                                                    .fg(app.theme.text_secondary)
                                                    .add_modifier(Modifier::ITALIC),
                                            ));
                                        }
                                    }
                                    (None, None) => {
                                        spans.push(Span::styled(" updated", Style::default().fg(app.theme.text_muted)));
                                    }
                                }

                                text.push(Line::from(spans));
                            }
                        }

                    // Closing line for journal entry
                    // Available width = area.width - 4 (borders) - 1 (└ char)
                    let line_width = (area.width as usize).saturating_sub(5);
                    text.push(Line::from(vec![
                        Span::styled("└", Style::default().fg(app.theme.text_muted)),
                        Span::styled(
                            "─".repeat(line_width),
                            Style::default().fg(app.theme.text_muted),
                        ),
                    ]));
                }
            }
        } else if app.loading_issue {
            text.push(Line::from(""));
            text.push(Line::from(""));
            text.push(Line::from(vec![Span::styled(
                "Loading notes... ⟳",
                Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
            )]));
        }

        let title = if app.loading_issue {
            " Issue Detail - Loading... ⟳ "
        } else {
            " Issue Detail (j/k scroll, g/G top/bottom, r reply, Shift+O browser, ESC close) "
        };

        let content_height = text.len();

        // Store content height in app for scrolling calculations
        app.popup_content_height = content_height;

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.primary))
                    .title(title),
            )
            .scroll((app.popup_scroll as u16, 0))
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);

        // Draw scrollbar
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .thumb_symbol("▐");
        let mut scrollbar_state =
            ScrollbarState::new(content_height.saturating_sub(area.height as usize)).position(app.popup_scroll);
        f.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

// Placeholder for help and config - these will remain in separate modules
pub fn draw_help(f: &mut Frame, app: &App, main_area: Rect) {
    // Render help as a centered popup overlay (80% width, 90% height)
    let area = centered_rect(80, 90, main_area);
    
    // Clear the background
    f.render_widget(Clear, area);
    
    let help_text = vec![
        Line::from(vec![Span::styled(
            "Redmine TUI - Keyboard Shortcuts",
            Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Global",
            Style::default()
                .fg(app.theme.success)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(vec![
            Span::styled("  ? ", Style::default().fg(app.theme.warning)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("  c ", Style::default().fg(app.theme.warning)),
            Span::raw("Open configuration"),
        ]),
        Line::from(vec![
            Span::styled("  q ", Style::default().fg(app.theme.warning)),
            Span::raw("Quit application"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C ", Style::default().fg(app.theme.warning)),
            Span::raw("Force quit"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default()
                .fg(app.theme.success)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(vec![
            Span::styled("  h/l ", Style::default().fg(app.theme.warning)),
            Span::raw("Switch between Projects and Issues panes"),
        ]),
        Line::from(vec![
            Span::styled("  j/k ", Style::default().fg(app.theme.warning)),
            Span::raw("Navigate down/up in lists"),
        ]),
        Line::from(vec![
            Span::styled("  g/G ", Style::default().fg(app.theme.warning)),
            Span::raw("Jump to top/bottom of list"),
        ]),
        Line::from(vec![
            Span::styled("  Enter ", Style::default().fg(app.theme.warning)),
            Span::raw("Select project / Open issue detail"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Projects",
            Style::default()
                .fg(app.theme.success)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(vec![
            Span::styled("  / ", Style::default().fg(app.theme.warning)),
            Span::raw("Search/filter projects"),
        ]),
        Line::from(vec![
            Span::styled("  R ", Style::default().fg(app.theme.warning)),
            Span::raw("Refresh projects from server"),
        ]),
        Line::from(vec![
            Span::styled("  Space ", Style::default().fg(app.theme.warning)),
            Span::raw("Toggle project collapse/expand"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Issues",
            Style::default()
                .fg(app.theme.success)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(vec![
            Span::styled("  / ", Style::default().fg(app.theme.warning)),
            Span::raw("Search/filter issues"),
        ]),
        Line::from(vec![
            Span::styled("  R ", Style::default().fg(app.theme.warning)),
            Span::raw("Refresh issues from server"),
        ]),
        Line::from(vec![
            Span::styled("  s ", Style::default().fg(app.theme.warning)),
            Span::raw("Cycle sort order (updated, created, id, priority)"),
        ]),
        Line::from(vec![
            Span::styled("  f ", Style::default().fg(app.theme.warning)),
            Span::raw("Filter by assignee"),
        ]),
        Line::from(vec![
            Span::styled("  n ", Style::default().fg(app.theme.warning)),
            Span::raw("Create new issue in selected project"),
        ]),
        Line::from(vec![
            Span::styled("  o ", Style::default().fg(app.theme.warning)),
            Span::raw("Open selected issue in browser"),
        ]),
        Line::from(vec![
            Span::styled("  b/B ", Style::default().fg(app.theme.warning)),
            Span::raw("Toggle bulk operation mode"),
        ]),
        Line::from(vec![
            Span::styled("  z ", Style::default().fg(app.theme.warning)),
            Span::raw("Toggle maximize issues pane"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Bulk Operations (when enabled)",
            Style::default()
                .fg(app.theme.success)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(vec![
            Span::styled("  Space ", Style::default().fg(app.theme.warning)),
            Span::raw("Toggle issue selection"),
        ]),
        Line::from(vec![
            Span::styled("  a ", Style::default().fg(app.theme.warning)),
            Span::raw("Select all visible issues"),
        ]),
        Line::from(vec![
            Span::styled("  A ", Style::default().fg(app.theme.warning)),
            Span::raw("Clear all selections"),
        ]),
        Line::from(vec![
            Span::styled("  x ", Style::default().fg(app.theme.warning)),
            Span::raw("Open bulk edit form"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Issue Detail",
            Style::default()
                .fg(app.theme.success)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(vec![
            Span::styled("  j/k ", Style::default().fg(app.theme.warning)),
            Span::raw("Scroll down/up"),
        ]),
        Line::from(vec![
            Span::styled("  g/G ", Style::default().fg(app.theme.warning)),
            Span::raw("Jump to top/bottom"),
        ]),
        Line::from(vec![
            Span::styled("  r ", Style::default().fg(app.theme.warning)),
            Span::raw("Reply / Add comment to issue"),
        ]),
        Line::from(vec![
            Span::styled("  1-9 ", Style::default().fg(app.theme.warning)),
            Span::raw("View/open attachment ("),
            Span::styled("Shift+Number", Style::default().fg(app.theme.warning)),
            Span::raw(" = open in browser)"),
        ]),
        Line::from(vec![
            Span::styled("  [/] ", Style::default().fg(app.theme.warning)),
            Span::raw("Previous/next attachment page"),
        ]),
        Line::from(vec![
            Span::styled("  Shift+O ", Style::default().fg(app.theme.warning)),
            Span::raw("Open issue in web browser"),
        ]),
        Line::from(vec![
            Span::styled("  ESC ", Style::default().fg(app.theme.warning)),
            Span::raw("Close detail popup"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Forms (Create Issue / Reply)",
            Style::default()
                .fg(app.theme.success)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(vec![
            Span::styled("  Tab ", Style::default().fg(app.theme.warning)),
            Span::raw("Move to next field"),
        ]),
        Line::from(vec![
            Span::styled("  Shift+Tab ", Style::default().fg(app.theme.warning)),
            Span::raw("Move to previous field"),
        ]),
        Line::from(vec![
            Span::styled("  Enter ", Style::default().fg(app.theme.warning)),
            Span::raw("Open dropdown / Select option / New line (text)"),
        ]),
        Line::from(vec![
            Span::styled("  / ", Style::default().fg(app.theme.warning)),
            Span::raw("Search in dropdown (when focused)"),
        ]),
        Line::from(vec![
            Span::styled("  ESC ", Style::default().fg(app.theme.warning)),
            Span::raw("Cancel / Close form"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+S ", Style::default().fg(app.theme.warning)),
            Span::raw("Submit form"),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.primary))
                .title(" Help (? to close) "),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

pub fn draw_config(f: &mut Frame, app: &App, area: Rect) {
    // Implementation from old popups.rs lines 1054-1405
    // Keep here or move to separate file later
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(13), // Logo and subtitle with spacing
            Constraint::Length(3),  // URL field
            Constraint::Length(3),  // API Key field
            Constraint::Length(17), // Theme selector (list of all themes)
            Constraint::Length(3),  // Exclude subprojects checkbox
            Constraint::Min(1),     // Empty space
            Constraint::Length(3),  // Instructions (at bottom)
        ])
        .margin(1)
        .split(area);

    // MineCLI Logo with subtitle
    let logo_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" ███╗   ███╗██╗███╗   ██╗███████╗ ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("  ██████╗ ██╗     ██╗", Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" ████╗ ████║██║████╗  ██║██╔════╝ ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("  ██╔════╝██║     ██║", Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" ██╔████╔██║██║██╔██╗ ██║█████╗   ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("  ██║     ██║     ██║", Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" ██║╚██╔╝██║██║██║╚██╗██║██╔══╝   ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("  ██║     ██║     ██║", Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" ██║ ╚═╝ ██║██║██║ ╚████║███████╗ ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("  ╚██████╗███████╗██║", Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" ╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚══════╝ ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("   ╚═════╝╚══════╝╚═╝", Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""), // 1 line above subtitle
        Line::from(vec![
            Span::styled("                 Redmine Terminal User Interface ", Style::default().fg(app.theme.text_secondary).add_modifier(Modifier::ITALIC)),
            Span::styled("v0.1.0", Style::default().fg(app.theme.text_muted).add_modifier(Modifier::ITALIC)),
        ]),
        Line::from(""), // 1 line below subtitle
        Line::from(""), // 2 lines below subtitle
        Line::from(""), // 3 lines below subtitle
        Line::from(""), // 4 lines below subtitle
    ];

    let logo_paragraph = Paragraph::new(logo_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(logo_paragraph, chunks[0]);

    let url_focused = matches!(app.input_mode, InputMode::Normal) && app.config_focused_field == 0;
    let api_key_focused = matches!(app.input_mode, InputMode::Normal) && app.config_focused_field == 1;
    let theme_focused = matches!(app.input_mode, InputMode::Normal) && app.config_focused_field == 2;
    let exclude_subprojects_focused = matches!(app.input_mode, InputMode::Normal) && app.config_focused_field == 3;

    let url_editing = matches!(app.input_mode, InputMode::EditingUrl);
    let api_key_editing = matches!(app.input_mode, InputMode::EditingApiKey);

    // Redmine URL field
    let url_border_color = if url_editing || url_focused {
        app.theme.warning
    } else if app.config.redmine_url.is_empty() {
        app.theme.error
    } else {
        app.theme.text_secondary
    };

    let url_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(url_border_color))
        .title(if url_editing {
            " Redmine URL * (Editing) "
        } else {
            " Redmine URL * "
        });

    let url_text = if url_editing {
        // When editing, show the input buffer
        app.url_input.clone()
    } else {
        app.config.redmine_url.clone()
    };

    let url_paragraph = Paragraph::new(url_text.clone())
        .style(Style::default().fg(if app.config.redmine_url.is_empty() {
            app.theme.text_muted
        } else {
            app.theme.text
        }))
        .block(url_block);
    f.render_widget(url_paragraph, chunks[1]);

    // Show cursor when editing URL
    if url_editing {
        let cursor_x = chunks[1].x + 1 + app.url_input.len() as u16;
        let cursor_y = chunks[1].y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }

    // API Key field
    let api_key_border_color = if api_key_editing || api_key_focused {
        app.theme.warning
    } else if app.config.api_key.is_empty() {
        app.theme.error
    } else {
        app.theme.text_secondary
    };

    let api_key_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(api_key_border_color))
        .title(if api_key_editing {
            " API Key * (Editing) "
        } else {
            " API Key * "
        });

    let api_key_display = if api_key_editing {
        // When editing, show the input buffer
        app.api_key_input.clone()
    } else {
        app.config.api_key.clone()
    };

    let api_key_paragraph = Paragraph::new(api_key_display.clone())
        .style(Style::default().fg(if app.config.api_key.is_empty() {
            app.theme.text_muted
        } else {
            app.theme.text
        }))
        .block(api_key_block);
    f.render_widget(api_key_paragraph, chunks[2]);

    // Show cursor when editing API key
    if api_key_editing {
        let cursor_x = chunks[2].x + 1 + app.api_key_input.len() as u16;
        let cursor_y = chunks[2].y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }

    // Theme selector - show all themes as a list
    let theme_border_color = if theme_focused {
        app.theme.warning
    } else {
        app.theme.text_secondary
    };

    let theme_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme_border_color))
        .title(" Theme ");

    let themes = crate::theme::ThemeName::all();
    let mut theme_lines: Vec<Line> = Vec::new();
    
    for (i, theme) in themes.iter().enumerate() {
        let is_selected = i == app.theme_selector_index;
        let theme_name = theme.to_string();
        
        let line = if is_selected && theme_focused {
            // Selected and focused - show with highlight
            Line::from(vec![
                Span::styled("▶ ", Style::default().fg(app.theme.success).add_modifier(Modifier::BOLD)),
                Span::styled(theme_name, Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)),
            ])
        } else if is_selected {
            // Selected but not focused
            Line::from(vec![
                Span::styled("▶ ", Style::default().fg(app.theme.text_secondary)),
                Span::styled(theme_name, Style::default().fg(app.theme.text)),
            ])
        } else {
            // Not selected
            Line::from(vec![
                Span::raw("  "),
                Span::styled(theme_name, Style::default().fg(app.theme.text_muted)),
            ])
        };
        
        theme_lines.push(line);
    }

    let theme_paragraph = Paragraph::new(theme_lines)
        .block(theme_block);
    f.render_widget(theme_paragraph, chunks[3]);

    // Exclude subprojects checkbox
    let exclude_subprojects_border_color = if exclude_subprojects_focused {
        app.theme.warning
    } else {
        app.theme.text_secondary
    };

    let exclude_subprojects_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(exclude_subprojects_border_color))
        .title(" Exclude Subproject Issues ");

    let checkbox_symbol = if app.config.exclude_subprojects {
        "[✓]"
    } else {
        "[ ]"
    };

    let exclude_subprojects_text = format!("{} Only load issues from selected project (exclude subprojects)", checkbox_symbol);

    let exclude_subprojects_paragraph = Paragraph::new(exclude_subprojects_text)
        .style(Style::default().fg(app.theme.text))
        .block(exclude_subprojects_block);
    f.render_widget(exclude_subprojects_paragraph, chunks[4]);

    // Instructions at the bottom
    let instructions = if url_editing || api_key_editing {
        "Editing mode: Type to input, ESC to finish editing"
    } else {
        "Tab/Shift+Tab: Navigate fields | ↑/↓ or j/k: Select theme | Enter: Edit/Save | ESC: Cancel"
    };

    let instructions_paragraph = Paragraph::new(instructions)
        .style(Style::default().fg(app.theme.text_muted))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(instructions_paragraph, chunks[6]);
}
