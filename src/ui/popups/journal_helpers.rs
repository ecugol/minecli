use crate::app::App;
use crate::redmine::JournalDetail;

/// Format a journal detail entry for display
pub fn format_journal_detail(app: &App, detail: &JournalDetail) -> (String, Option<String>, Option<String>) {
    // Determine field display name based on property
    let field_display = match detail.property.as_str() {
        "attr" => {
            // Standard attribute - use name with better formatting
            match detail.name.as_str() {
                "status_id" => "Status".to_string(),
                "assigned_to_id" => "Assignee".to_string(),
                "priority_id" => "Priority".to_string(),
                "tracker_id" => "Tracker".to_string(),
                "done_ratio" => "Progress".to_string(),
                "estimated_hours" => "Estimated Time".to_string(),
                "due_date" => "Due Date".to_string(),
                "start_date" => "Start Date".to_string(),
                "subject" => "Subject".to_string(),
                "description" => "Description".to_string(),
                "category_id" => "Category".to_string(),
                _ => detail.name.clone(),
            }
        }
        "cf" => {
            // Custom field - look up the field name
            if let Ok(field_id) = detail.name.parse::<u64>() {
                if let Some(issue) = &app.current_issue {
                    if let Some(field) = issue.custom_fields.iter().find(|f| f.id == field_id) {
                        field.name.clone()
                    } else {
                        format!("Custom Field {}", detail.name)
                    }
                } else {
                    format!("Custom Field {}", detail.name)
                }
            } else {
                detail.name.clone()
            }
        }
        "attachment" => {
            // Attachment change - show the actual filename
            if let Some(filename) = detail.new_value.as_ref().or(detail.old_value.as_ref()) {
                format!("File {}", filename)
            } else {
                "File".to_string()
            }
        }
        _ => detail.name.clone(),
    };

    // Convert values to verbose format
    let old_verbose = detail
        .old_value
        .as_ref()
        .map(|v| convert_field_value_to_verbose(app, &detail.name, &detail.property, v));
    let new_verbose = detail
        .new_value
        .as_ref()
        .map(|v| convert_field_value_to_verbose(app, &detail.name, &detail.property, v));

    (field_display, old_verbose, new_verbose)
}

/// Convert field value ID to verbose name
fn convert_field_value_to_verbose(app: &App, field_name: &str, property: &str, value: &str) -> String {
    // Handle attachments specially
    if property == "attachment" {
        return value.to_string();
    }

    match field_name.to_lowercase().as_str() {
        "status" | "status_id" => {
            // Value might already be the name or could be an ID
            if let Ok(id) = value.parse::<u64>() {
                app.statuses
                    .iter()
                    .find(|s| s.id == id)
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| value.to_string())
            } else {
                value.to_string()
            }
        }
        "priority" | "priority_id" => {
            if let Ok(id) = value.parse::<u64>() {
                app.priorities
                    .iter()
                    .find(|p| p.id == id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| value.to_string())
            } else {
                value.to_string()
            }
        }
        "tracker" | "tracker_id" => {
            if let Ok(id) = value.parse::<u64>() {
                app.trackers
                    .iter()
                    .find(|t| t.id == id)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| value.to_string())
            } else {
                value.to_string()
            }
        }
        "assigned_to" | "assigned_to_id" => {
            if let Ok(id) = value.parse::<u64>() {
                // Try to find user in cached list
                if let Some(user) = app.users.iter().find(|u| u.id == id) {
                    let full_name = format!("{} {}", user.firstname, user.lastname).trim().to_string();
                    if !full_name.is_empty() {
                        return full_name;
                    }
                }
                
                // Try to find in current issue's author/assignee
                if let Some(issue) = &app.current_issue {
                    if issue.author.id == id {
                        return issue.author.name.clone();
                    }
                    if let Some(assignee) = &issue.assigned_to {
                        if assignee.id == id {
                            return assignee.name.clone();
                        }
                    }
                }
                
                // Fallback to showing ID
                format!("User #{}", id)
            } else {
                // Value might already be the name
                value.to_string()
            }
        }
        "category" | "category_id" => {
            if let Ok(id) = value.parse::<u64>() {
                app.categories
                    .iter()
                    .find(|c| c.id == id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| value.to_string())
            } else {
                value.to_string()
            }
        }
        "done_ratio" => {
            format!("{}%", value)
        }
        "estimated_hours" => {
            if let Ok(hours) = value.parse::<f32>() {
                format!("{:.1}h", hours)
            } else {
                value.to_string()
            }
        }
        _ => value.to_string(),
    }
}

/// Get color for field value based on field type
pub fn get_field_color(app: &App, field_name: &str, value: Option<&str>) -> ratatui::style::Color {
    match field_name.to_lowercase().as_str() {
        "status" | "status_id" => {
            // Color based on status name
            if let Some(status) = value {
                match status.to_lowercase().as_str() {
                    s if s.contains("new") || s.contains("open") => app.theme.info,
                    s if s.contains("progress") || s.contains("assigned") => app.theme.warning,
                    s if s.contains("resolved") || s.contains("closed") || s.contains("done") => app.theme.success,
                    s if s.contains("rejected") || s.contains("invalid") => app.theme.error,
                    _ => app.theme.accent,
                }
            } else {
                app.theme.text_secondary
            }
        }
        "priority" | "priority_id" => {
            if let Some(priority) = value {
                match priority.to_lowercase().as_str() {
                    s if s.contains("low") => app.theme.success,
                    s if s.contains("normal") => app.theme.info,
                    s if s.contains("high") || s.contains("urgent") => app.theme.warning,
                    s if s.contains("immediate") => app.theme.error,
                    _ => app.theme.accent,
                }
            } else {
                app.theme.text_secondary
            }
        }
        "done_ratio" => {
            if let Some(ratio_str) = value {
                if let Some(num) = ratio_str.strip_suffix('%') {
                    if let Ok(ratio) = num.parse::<u32>() {
                        match ratio {
                            0..=25 => app.theme.error,
                            26..=50 => app.theme.warning,
                            51..=75 => app.theme.info,
                            76..=99 => app.theme.accent,
                            100 => app.theme.success,
                            _ => app.theme.text_secondary,
                        }
                    } else {
                        app.theme.text_secondary
                    }
                } else {
                    app.theme.text_secondary
                }
            } else {
                app.theme.text_secondary
            }
        }
        "assigned_to" | "assigned_to_id" => app.theme.warning,
        "tracker" | "tracker_id" => app.theme.info,
        "category" | "category_id" => app.theme.accent,
        _ => app.theme.text_secondary,
    }
}
