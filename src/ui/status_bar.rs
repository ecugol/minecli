use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, InputMode, Pane, Screen};

/// Get context-aware help text based on current app state
fn get_contextual_help(app: &App) -> Vec<(String, String)> {
    // Returns vector of (key, description) tuples
    match app.input_mode {
        InputMode::Normal => {
            if app.show_issue_popup {
                // Issue popup is showing
                vec![
                    ("j/k".to_string(), "Scroll".to_string()),
                    ("J/K".to_string(), "Next/Prev Issue".to_string()),
                    ("g/G".to_string(), "Top/Bottom".to_string()),
                    ("r".to_string(), "Reply".to_string()),
                    ("O".to_string(), "Open in Browser".to_string()),
                    ("1-9".to_string(), "View Attachment".to_string()),
                    ("[/]".to_string(), "Prev/Next Page".to_string()),
                ]
            } else {
                match (app.screen, app.focused_pane) {
                    (Screen::Main, Pane::Projects) => vec![
                        ("Enter".to_string(), "Select".to_string()),
                        ("j/k".to_string(), "Navigate".to_string()),
                        ("Space".to_string(), "Collapse".to_string()),
                        ("/".to_string(), "Search".to_string()),
                        ("P".to_string(), "Refresh".to_string()),
                        ("l".to_string(), "→Issues".to_string()),
                    ],
                    (Screen::Main, Pane::Issues) if app.selected_project.is_some() => {
                        let mut help = vec![
                            ("j/k".to_string(), "Navigate".to_string()),
                            ("n".to_string(), "New".to_string()),
                            ("/".to_string(), "Search".to_string()),
                            ("I".to_string(), "Refresh".to_string()),
                            ("h".to_string(), "←Projects".to_string()),
                        ];
                        if !app.filtered_issues.is_empty() {
                            help.insert(0, ("Enter".to_string(), "View".to_string()));
                            if app.bulk_operation_mode {
                                help.insert(1, ("Space".to_string(), "Select".to_string()));
                                help.insert(2, ("a/A".to_string(), "All/None".to_string()));
                                help.insert(3, ("x".to_string(), "Edit".to_string()));
                                help.insert(4, ("b".to_string(), "Exit Bulk".to_string()));
                            } else {
                                help.push(("s".to_string(), "Sort".to_string()));
                                help.push(("g".to_string(), "Group".to_string()));
                                help.push(("b".to_string(), "Bulk".to_string()));
                                help.push(("m".to_string(), "My Issues".to_string()));
                                help.push(("z".to_string(), "Maximize".to_string()));
                            }
                        }
                        help
                    },
                    (Screen::Main, Pane::Issues) => vec![
                        ("h".to_string(), "←Projects".to_string()),
                    ],
                    (Screen::Config, _) => vec![
                        ("Tab".to_string(), "Next".to_string()),
                        ("Enter".to_string(), "Edit/Toggle".to_string()),
                        ("ESC".to_string(), "Back".to_string()),
                    ],
                    _ => vec![],
                }
            }
        },
        InputMode::Searching => vec![
            ("Enter".to_string(), "Apply".to_string()),
            ("ESC".to_string(), "Cancel".to_string()),
        ],
        InputMode::CreatingIssue | InputMode::ReplyingToIssue => vec![
            ("Tab".to_string(), "Next Field".to_string()),
            ("@".to_string(), "Attach".to_string()),
            ("#".to_string(), "Manage".to_string()),
            ("Ctrl+S".to_string(), "Save".to_string()),
            ("ESC".to_string(), "Cancel".to_string()),
        ],
        InputMode::BulkEditing => vec![
            ("Tab".to_string(), "Next Field".to_string()),
            ("Ctrl+S".to_string(), "Save".to_string()),
            ("ESC".to_string(), "Cancel".to_string()),
        ],
        InputMode::AddingAttachment => vec![
            ("Enter".to_string(), "Select".to_string()),
            ("h".to_string(), "Hidden".to_string()),
            ("ESC".to_string(), "Cancel".to_string()),
        ],
        InputMode::ManagingAttachments => vec![
            ("j/k".to_string(), "Navigate".to_string()),
            ("d/Del".to_string(), "Remove".to_string()),
            ("ESC".to_string(), "Close".to_string()),
        ],
        _ => vec![],
    }
}

/// Get shortened help text for narrow screens
fn get_compact_help(app: &App) -> Vec<(String, String)> {
    match app.input_mode {
        InputMode::Normal => {
            if app.show_issue_popup {
                vec![
                    ("j/k".to_string(), "↕".to_string()),
                    ("J/K".to_string(), "Issue".to_string()),
                    ("r".to_string(), "Reply".to_string()),
                    ("1-9".to_string(), "Attach".to_string()),
                ]
            } else {
                match (app.screen, app.focused_pane) {
                    (Screen::Main, Pane::Projects) => vec![
                        ("↵".to_string(), "Select".to_string()),
                        ("j/k".to_string(), "↕".to_string()),
                        ("/".to_string(), "Search".to_string()),
                        ("l".to_string(), "→".to_string()),
                    ],
                    (Screen::Main, Pane::Issues) if !app.filtered_issues.is_empty() => {
                        if app.bulk_operation_mode {
                            vec![
                                ("↵".to_string(), "View".to_string()),
                                ("Spc".to_string(), "☑".to_string()),
                                ("x".to_string(), "Edit".to_string()),
                                ("b".to_string(), "Exit".to_string()),
                            ]
                        } else {
                            vec![
                                ("↵".to_string(), "View".to_string()),
                                ("n".to_string(), "New".to_string()),
                                ("s".to_string(), "Sort".to_string()),
                                ("h".to_string(), "←".to_string()),
                            ]
                        }
                    },
                    (Screen::Main, Pane::Issues) if app.selected_project.is_some() => vec![
                        ("n".to_string(), "New".to_string()),
                        ("h".to_string(), "←".to_string()),
                    ],
                    (Screen::Main, Pane::Issues) => vec![
                        ("h".to_string(), "←Projects".to_string()),
                    ],
                    (Screen::Config, _) => vec![
                        ("Tab".to_string(), "Next".to_string()),
                        ("↵".to_string(), "Edit".to_string()),
                    ],
                    _ => vec![],
                }
            }
        },
        InputMode::Searching => vec![
            ("↵".to_string(), "Apply".to_string()),
            ("Esc".to_string(), "Cancel".to_string()),
        ],
        InputMode::CreatingIssue | InputMode::ReplyingToIssue => vec![
            ("Tab".to_string(), "Next".to_string()),
            ("@".to_string(), "File".to_string()),
            ("^S".to_string(), "Save".to_string()),
            ("Esc".to_string(), "Cancel".to_string()),
        ],
        InputMode::BulkEditing => vec![
            ("^S".to_string(), "Save".to_string()),
            ("Esc".to_string(), "Cancel".to_string()),
        ],
        InputMode::AddingAttachment => vec![
            ("↵".to_string(), "Select".to_string()),
            ("h".to_string(), "Hidden".to_string()),
        ],
        InputMode::ManagingAttachments => vec![
            ("j/k".to_string(), "↕".to_string()),
            ("d".to_string(), "Remove".to_string()),
        ],
        _ => vec![],
    }
}

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    // Helper function to generate current selection line
    let get_selection_line = || -> Line {
        let mut spans = vec![];

        // Show current project with last activity (use helper to get project at cursor)
        if let Some(project) = app.get_project_at_cursor() {
            spans.push(Span::styled("Project: ", Style::default().fg(app.theme.text_muted)));
            spans.push(Span::styled(
                &project.name,
                Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD),
            ));

            // Show last activity date
            if let Some(last_activity) = project.last_issue_activity.as_ref()
                .or(project.updated_on.as_ref())
                .or(project.created_on.as_ref())
            {
                let activity_str = last_activity.format("%Y-%m-%d %H:%M").to_string();
                spans.push(Span::raw(" ("));
                spans.push(Span::styled(
                    activity_str,
                    Style::default().fg(app.theme.accent),
                ));
                spans.push(Span::raw(")"));
            }
        } else {
            spans.push(Span::styled("No project selected", Style::default().fg(app.theme.text_muted)));
        }

        // Show current issue if any (using helper that handles grouped mode)
        if let Some(issue) = app.get_issue_at_cursor() {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled("Issue: ", Style::default().fg(app.theme.text_muted)));
            spans.push(Span::styled(
                format!("#{}", issue.id),
                Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));

            // Truncate subject if too long
            let subject = if issue.subject.len() > 60 {
                format!("{}...", issue.subject.chars().take(57).collect::<String>())
            } else {
                issue.subject.clone()
            };
            spans.push(Span::raw(subject));
        }

        Line::from(spans)
    };

    // Helper function to generate sync info line
    let get_sync_line = || -> Line {
        let mut line2_spans = vec![];

        // Projects sync time
        if let Some(sync_time) = &app.last_projects_sync {
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(*sync_time);

            let time_ago = if duration.num_minutes() < 1 {
                "just now".to_string()
            } else if duration.num_minutes() < 60 {
                format!("{}m ago", duration.num_minutes())
            } else if duration.num_hours() < 24 {
                format!("{}h ago", duration.num_hours())
            } else {
                format!("{}d ago", duration.num_days())
            };

            line2_spans.extend(vec![
                Span::raw("Projects synced: "),
                Span::styled(time_ago, Style::default().fg(app.theme.text_muted)),
            ]);
        } else {
            line2_spans.extend(vec![
                Span::raw("Projects synced: "),
                Span::styled("never", Style::default().fg(app.theme.text_muted)),
            ]);
        }

        line2_spans.push(Span::raw(" | "));

        // Issues sync time - per-project
        if let Some(project) = &app.selected_project {
            if let Some(sync_time) = &project.last_issues_sync {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(*sync_time);

                let time_ago = if duration.num_minutes() < 1 {
                    "just now".to_string()
                } else if duration.num_minutes() < 60 {
                    format!("{}m ago", duration.num_minutes())
                } else if duration.num_hours() < 24 {
                    format!("{}h ago", duration.num_hours())
                } else {
                    format!("{}d ago", duration.num_days())
                };

                line2_spans.extend(vec![
                    Span::raw("Issues synced: "),
                    Span::styled(time_ago, Style::default().fg(app.theme.text_muted)),
                ]);
            } else {
                line2_spans.extend(vec![
                    Span::raw("Issues synced: "),
                    Span::styled("never", Style::default().fg(app.theme.text_muted)),
                ]);
            }
        } else {
            line2_spans.extend(vec![
                Span::raw("Issues synced: "),
                Span::styled("no project", Style::default().fg(app.theme.text_muted)),
            ]);
        }

        Line::from(line2_spans)
    };

    let status_text = if app.input_mode == InputMode::Searching {
        vec![
            Line::from(vec![
                Span::styled(
                    "Search: /",
                    Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
                ),
                Span::styled(&app.search_query, Style::default().fg(app.theme.text)),
                Span::styled("█", Style::default().fg(app.theme.warning)),
                Span::raw("  "),
                Span::styled("Enter", Style::default().fg(app.theme.success)),
                Span::raw(" to apply | "),
                Span::styled("ESC", Style::default().fg(app.theme.error)),
                Span::raw(" to cancel"),
            ]),
            get_sync_line(),
        ]
    } else if let Some(error) = &app.error_message {
        // Truncate error to fit in status bar, add hint to view full error
        let truncated = if error.len() > 100 {
            format!(
                "{}... (Press 'e' for full error)",
                &error.chars().take(97).collect::<String>()
            )
        } else {
            error.clone()
        };
        vec![
            Line::from(vec![
                Span::styled(
                    "✗ Error: ",
                    Style::default().fg(app.theme.error).add_modifier(Modifier::BOLD),
                ),
                Span::styled(truncated, Style::default().fg(app.theme.error)),
            ]),
            get_sync_line(),
        ]
    } else if let Some(status) = &app.status_message {
        // Show status message but also show contextual help if in a special mode
        if app.show_issue_popup || app.show_create_issue_form || app.bulk_edit_form.is_some() 
            || app.update_issue_form.is_some() || app.input_mode == InputMode::AddingAttachment
            || app.input_mode == InputMode::ManagingAttachments {
            // In special modes, show compact help instead of full status
            let use_compact = area.width < 100;
            let contextual_help = if use_compact {
                get_compact_help(app)
            } else {
                get_contextual_help(app)
            };
            
            let mut help_spans = vec![];
            for (i, (key, desc)) in contextual_help.into_iter().enumerate() {
                if i > 0 {
                    help_spans.push(Span::raw("  "));
                }
                help_spans.push(Span::styled(
                    key,
                    Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
                ));
                help_spans.push(Span::raw(" "));
                help_spans.push(Span::raw(desc));
            }
            
            // Add minimal global shortcuts
            if !help_spans.is_empty() {
                help_spans.push(Span::raw("  "));
            }
            help_spans.extend(vec![
                Span::styled("?", Style::default().fg(app.theme.primary)),
                Span::raw(" Help  "),
                Span::styled("q", Style::default().fg(app.theme.primary)),
                Span::raw(" Quit"),
            ]);
            
            vec![
                Line::from(help_spans),
                Line::from(vec![Span::styled(
                    format!("✓ {}", status),
                    Style::default().fg(app.theme.success),
                )]),
                get_sync_line(),
            ]
        } else {
            // Normal status message display
            vec![
                Line::from(vec![Span::styled(
                    format!("✓ {}", status),
                    Style::default().fg(app.theme.success),
                )]),
                get_sync_line(),
            ]
        }
    } else {
        // Choose between full and compact help based on terminal width
        let use_compact = area.width < 100;
        let contextual_help = if use_compact {
            get_compact_help(app)
        } else {
            get_contextual_help(app)
        };
        
        // Build help line with context-aware shortcuts
        let mut help_spans = vec![];
        for (i, (key, desc)) in contextual_help.into_iter().enumerate() {
            if i > 0 {
                help_spans.push(Span::raw("  "));
            }
            help_spans.push(Span::styled(
                key,
                Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
            ));
            help_spans.push(Span::raw(" "));
            help_spans.push(Span::raw(desc));
        }
        
        // Add global shortcuts at the end (compact version for narrow screens)
        if !help_spans.is_empty() {
            help_spans.push(Span::raw("  "));
        }
        if use_compact {
            help_spans.extend(vec![
                Span::styled("?", Style::default().fg(app.theme.primary)),
                Span::raw(" Help  "),
                Span::styled("q", Style::default().fg(app.theme.primary)),
                Span::raw(" Quit"),
            ]);
        } else {
            help_spans.extend(vec![
                Span::styled("?", Style::default().fg(app.theme.primary)),
                Span::raw(" Help  "),
                Span::styled("c", Style::default().fg(app.theme.primary)),
                Span::raw(" Config  "),
                Span::styled("q", Style::default().fg(app.theme.primary)),
                Span::raw(" Quit"),
            ]);
        }

        vec![
            Line::from(help_spans),
            get_sync_line(),
        ]
    };

    let status_bar = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.text_muted)),
        )
        .alignment(Alignment::Center);

    f.render_widget(status_bar, area);
}
