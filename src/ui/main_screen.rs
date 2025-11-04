use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use super::widgets::get_loading_spinner;
use crate::app::{App, Pane};

pub fn draw_main_screen(f: &mut Frame, app: &mut App, area: Rect) {
    if app.selected_project.is_none() {
        // No project selected - show only projects pane
        draw_projects_pane(f, app, area);
    } else {
        // Project selected - split into two panes (or maximize issues pane)
        if app.issues_pane_maximized {
            // Show only issues pane when maximized
            draw_issues_pane(f, app, area);
        } else {
            // Normal split view
            let panes = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(area);

            draw_projects_pane(f, app, panes[0]);
            draw_issues_pane(f, app, panes[1]);
        }
    }
}

fn draw_projects_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_pane == Pane::Projects;

    // Build a tree structure from flat list
    let mut root_projects = Vec::new();
    let mut child_map: std::collections::HashMap<u64, Vec<&crate::redmine::Project>> = std::collections::HashMap::new();

    // Separate root projects and build child map
    for project in &app.filtered_projects {
        if let Some(parent) = &project.parent {
            child_map.entry(parent.id).or_default().push(project);
        } else {
            root_projects.push(project);
        }
    }

    // Build display list with tree structure
    let mut display_list = Vec::new();
    let mut display_index = 0;

    fn add_project_tree<'a>(
        project: &'a crate::redmine::Project,
        child_map: &std::collections::HashMap<u64, Vec<&'a crate::redmine::Project>>,
        collapsed_map: &std::collections::HashMap<u64, bool>,
        display_list: &mut Vec<(usize, &'a crate::redmine::Project, usize)>, // (display_index, project, depth)
        display_index: &mut usize,
        depth: usize,
    ) {
        display_list.push((*display_index, project, depth));
        *display_index += 1;

        // Add children if this project is not collapsed
        let is_collapsed = collapsed_map.get(&project.id).copied().unwrap_or(false);
        if !is_collapsed {
            if let Some(children) = child_map.get(&project.id) {
                for child in children {
                    add_project_tree(child, child_map, collapsed_map, display_list, display_index, depth + 1);
                }
            }
        }
    }

    for project in root_projects {
        add_project_tree(
            project,
            &child_map,
            &app.projects_collapsed,
            &mut display_list,
            &mut display_index,
            0,
        );
    }

    let items: Vec<ListItem> = display_list
        .iter()
        .enumerate()
        .map(|(i, (_, project, depth))| {
            let is_selected = Some(project.id) == app.selected_project.as_ref().map(|p| p.id);

            // Check if project has children
            let has_children = child_map.contains_key(&project.id);
            let is_collapsed = app.projects_collapsed.get(&project.id).copied().unwrap_or(false);

            // Build fold indicator
            let fold_indicator = if has_children {
                if is_collapsed {
                    "▶"
                } else {
                    "▼"
                }
            } else {
                " "
            };

            // Build indentation
            let indent = "  ".repeat(*depth);

            let content = format!(
                "{}{} {}",
                indent,
                fold_indicator,
                project.name
            );

            let style = if i == app.projects_list_state && is_focused {
                // Cursor is on this project AND pane is focused
                Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)
            } else if i == app.projects_list_state && is_selected {
                // Cursor is on selected project but pane not focused
                Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD)
            } else if i == app.projects_list_state {
                // Cursor is on this project but pane not focused and not selected
                Style::default()
                    .fg(app.theme.text_secondary)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                // This is the selected project (even when focus is on issues pane)
                // Use accent color + bold to make it very visible
                Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD)
            } else if is_focused {
                // Not cursor, not selected, but pane is focused - normal color
                Style::default().fg(app.theme.text)
            } else {
                // Pane not focused, not selected - muted color
                Style::default().fg(app.theme.text_muted)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    // Add loader for projects
    let project_loader = if app.loading && app.total_issues == 0 {
        format!(" {} Loading...", get_loading_spinner())
    } else {
        String::new()
    };

    let title = if !app.project_filter.is_empty() {
        format!(
            "Projects ({}/{}{}) [Filter: {}]",
            app.filtered_projects.len(),
            app.total_projects,
            project_loader,
            app.project_filter
        )
    } else {
        format!("Projects ({}{})", app.filtered_projects.len(), project_loader)
    };

    let border_style = if is_focused {
        Style::default().fg(app.theme.border_focused)
    } else {
        Style::default().fg(app.theme.border)
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title),
    );

    let mut list_state = ListState::default();
    list_state.select(Some(app.projects_list_state));
    f.render_stateful_widget(list, area, &mut list_state);

    // Draw scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_symbol("▐");
    let mut scrollbar_state = ScrollbarState::new(display_list.len()).position(app.projects_list_state);
    f.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn draw_issues_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_pane == Pane::Issues;

    let items: Vec<ListItem> = if app.group_issues_by_status {
        // Grouped mode: organize by status
        let mut grouped_items = Vec::new();
        let mut display_index = 0;
        let mut status_order: Vec<String> = Vec::new();

        // Collect unique statuses in order
        for issue in &app.filtered_issues {
            if !status_order.contains(&issue.status.name) {
                status_order.push(issue.status.name.clone());
            }
        }

        // Define custom status order: In progress > Feedback > New > Resolved > Closed
        let status_priority = |status: &str| -> u32 {
            match status.to_lowercase().as_str() {
                s if s.contains("in progress") || s.contains("progress") => 1,
                s if s.contains("feedback") => 2,
                s if s.contains("new") => 3,
                s if s.contains("resolved") => 4,
                s if s.contains("closed") => 5,
                _ => 99, // Unknown statuses go to the end
            }
        };

        // Sort statuses by custom priority order
        status_order.sort_by_key(|s| status_priority(s));

        // Get the current status group to highlight its header
        let current_group = app.get_current_status_group();

        for status_name in status_order {
            let is_collapsed = app.status_groups_collapsed.get(&status_name).copied().unwrap_or(false);
            let issue_count = app
                .filtered_issues
                .iter()
                .filter(|i| i.status.name == status_name)
                .count();
            let status_color = app.theme.get_status_color(&status_name);

            // Check if this header's group is currently selected (cursor on header or any issue)
            let is_current_group = current_group.as_ref().map(|g| g == &status_name).unwrap_or(false);

            // Status header
            let collapse_indicator = if is_collapsed { "▶" } else { "▼" };

            // Apply highlight with background color if this is the current group
            let header_style = if is_current_group && is_focused {
                Style::default()
                    .fg(app.theme.background)
                    .bg(app.theme.warning)
                    .add_modifier(Modifier::BOLD)
            } else if is_current_group {
                Style::default()
                    .fg(app.theme.background)
                    .bg(app.theme.text_secondary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let header_content = if is_current_group {
                // When highlighted, use a solid style for the whole line
                vec![Span::styled(
                    format!("{} {} ({})", collapse_indicator, status_name, issue_count),
                    header_style,
                )]
            } else {
                // Normal rendering with color-coded status
                vec![
                    Span::styled(
                        format!("{} ", collapse_indicator),
                        Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{} ({})", status_name, issue_count),
                        Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                    ),
                ]
            };

            grouped_items.push(ListItem::new(Line::from(header_content)));
            display_index += 1;

            // Issues under this status (if not collapsed)
            if !is_collapsed {
                for issue in app.filtered_issues.iter().filter(|i| i.status.name == status_name) {
                    let priority_color = app.theme.get_priority_color(&issue.priority.name);
                    let assignee_str = issue
                        .assigned_to
                        .as_ref()
                        .map(|a| {
                            let name = a.name.chars().take(10).collect::<String>();
                            if a.name.len() > 10 {
                                format!("@{}… ", name)
                            } else {
                                format!("@{} ", name)
                            }
                        })
                        .unwrap_or_else(|| "(none) ".to_string());

                    // Check if issue is updated since last sync
                    let is_updated = app.is_issue_updated_since_last_sync(issue);
                    let update_marker = if is_updated { "*" } else { "" };

                    // Checkbox for bulk operations
                    let mut content = vec![Span::raw("  ")]; // Indent

                    if app.bulk_operation_mode {
                        let checkbox = if app.is_issue_selected(issue.id) {
                            "[✓] "
                        } else {
                            "[ ] "
                        };
                        content.push(Span::styled(
                            checkbox,
                            Style::default().fg(app.theme.success).add_modifier(Modifier::BOLD),
                        ));
                    }

                    // Determine if this issue is at cursor
                    let is_at_cursor = display_index == app.issues_list_state;
                    
                    // Issue number - bold and highlighted color when at cursor
                    let issue_num_style = if is_at_cursor {
                        if is_focused {
                            Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(app.theme.text_secondary).add_modifier(Modifier::BOLD)
                        }
                    } else {
                        Style::default().fg(app.theme.text_muted)
                    };

                    content.extend(vec![
                        Span::styled(
                            format!("#{:<6}{} ", issue.id, update_marker),
                            issue_num_style,
                        ),
                        Span::styled(
                            format!("{} ", issue.priority.name.chars().next().unwrap_or('N')),
                            Style::default().fg(priority_color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(assignee_str, Style::default().fg(app.theme.accent)),
                        Span::raw(&issue.subject),
                    ]);

                    let mut line_style = Style::default();
                    if is_at_cursor && is_focused {
                        line_style = line_style.fg(app.theme.warning).add_modifier(Modifier::BOLD);
                    } else if is_at_cursor {
                        line_style = line_style.fg(app.theme.text_secondary).add_modifier(Modifier::BOLD);
                    }

                    grouped_items.push(ListItem::new(Line::from(content)).style(line_style));
                    display_index += 1;
                }
            }
        }

        grouped_items
    } else {
        // Normal mode: flat list
        app.filtered_issues
            .iter()
            .enumerate()
            .map(|(i, issue)| {
                let status_color = app.theme.get_status_color(&issue.status.name);
                let priority_color = app.theme.get_priority_color(&issue.priority.name);
                let assignee_str = issue
                    .assigned_to
                    .as_ref()
                    .map(|a| {
                        let name = a.name.chars().take(10).collect::<String>();
                        if a.name.len() > 10 {
                            format!("@{}… ", name)
                        } else {
                            format!("@{} ", name)
                        }
                    })
                    .unwrap_or_else(|| "(none) ".to_string());

                // Check if issue is updated since last sync
                let is_updated = app.is_issue_updated_since_last_sync(issue);
                let update_marker = if is_updated { "*" } else { "" };

                // Checkbox for bulk operations
                let mut content = vec![];

                if app.bulk_operation_mode {
                    let checkbox = if app.is_issue_selected(issue.id) {
                        "[✓] "
                    } else {
                        "[ ] "
                    };
                    content.push(Span::styled(
                        checkbox,
                        Style::default().fg(app.theme.success).add_modifier(Modifier::BOLD),
                    ));
                }

                // Determine if this issue is at cursor
                let is_at_cursor = i == app.issues_list_state;
                
                // Issue number - bold and highlighted color when at cursor
                let issue_num_style = if is_at_cursor {
                    if is_focused {
                        Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(app.theme.text_secondary).add_modifier(Modifier::BOLD)
                    }
                } else {
                    Style::default().fg(app.theme.text_muted)
                };

                content.extend(vec![
                    Span::styled(
                        format!("#{:<6}{} ", issue.id, update_marker),
                        issue_num_style,
                    ),
                    Span::styled(format!("[{}] ", issue.status.name), Style::default().fg(status_color)),
                    Span::styled(
                        format!("{} ", issue.priority.name.chars().next().unwrap_or('N')),
                        Style::default().fg(priority_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(assignee_str, Style::default().fg(app.theme.accent)),
                    Span::raw(&issue.subject),
                ]);

                let mut line_style = Style::default();
                if is_at_cursor && is_focused {
                    line_style = line_style.fg(app.theme.warning).add_modifier(Modifier::BOLD);
                } else if is_at_cursor {
                    line_style = line_style.fg(app.theme.text_secondary).add_modifier(Modifier::BOLD);
                }

                ListItem::new(Line::from(content)).style(line_style)
            })
            .collect()
    };

    // Add loader animation - make it more visible
    let loader = if app.loading {
        format!(" {} Loading...", get_loading_spinner())
    } else {
        String::new()
    };

    let title = if let Some(project) = &app.selected_project {
        let sort_indicator = format!(" [Sort: {}]", app.issue_sort_order.as_str());
        let my_issues_indicator = if app.my_issues_filter { " [My Issues]" } else { "" };
        if !app.issue_filter.is_empty() {
            format!(
                "{}: Issues ({}/{}){}{}{} [Filter: {}]",
                project.name,
                app.filtered_issues.len(),
                app.total_issues,
                loader,
                sort_indicator,
                my_issues_indicator,
                app.issue_filter
            )
        } else {
            format!(
                "{}: Issues ({}){}{}{}",
                project.name,
                app.filtered_issues.len(),
                loader,
                sort_indicator,
                my_issues_indicator
            )
        }
    } else {
        "Issues (Select a project)".to_string()
    };

    let border_style = if is_focused {
        Style::default().fg(app.theme.border_focused)
    } else {
        Style::default().fg(app.theme.border)
    };

    // Get length before moving items
    let scrollbar_len = items.len();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title),
    );

    let mut list_state = ListState::default();
    list_state.select(Some(app.issues_list_state));
    f.render_stateful_widget(list, area, &mut list_state);

    // Draw scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(None)
        .thumb_symbol("▐");
    let mut scrollbar_state = ScrollbarState::new(scrollbar_len).position(app.issues_list_state);
    f.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}
