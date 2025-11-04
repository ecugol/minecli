use std::collections::HashMap;

use super::state::{App, IssueSortOrder};
use crate::form_field::FieldValue;
use crate::issue_form::IssueForm;
use crate::redmine::{Issue, Project};

impl App {
    pub fn apply_filters(&mut self) {
        // Load sync timestamps
        self.last_projects_sync = self.db.get_last_projects_sync().ok().flatten();

        // Query projects from database
        let filter = if self.project_filter.is_empty() {
            None
        } else {
            Some(self.project_filter.as_str())
        };

        // Get total count first (no filter)
        self.total_projects = self.db.get_projects(None).map(|p| p.len()).unwrap_or(0);

        self.filtered_projects = self.db.get_projects(filter).unwrap_or_else(|e| {
            self.error_message = Some(format!("Failed to query projects: {}", e));
            Vec::new()
        });

        // Reset selection if out of bounds
        if self.projects_list_state >= self.filtered_projects.len() {
            self.projects_list_state = self.filtered_projects.len().saturating_sub(1);
        }

        // Query issues from database with sorting and filtering built-in
        let project_id = self.selected_project.as_ref().map(|p| p.id);
        let issue_filter = if self.issue_filter.is_empty() {
            None
        } else {
            Some(self.issue_filter.as_str())
        };

        // Apply "my issues" filter if enabled
        let assigned_to_filter = if self.my_issues_filter {
            self.current_user_id
        } else {
            None
        };

        // Get total count first (no filter)
        self.total_issues = self
            .db
            .get_issues(project_id, self.issue_sort_order, None, None)
            .map(|i| i.len())
            .unwrap_or(0);

        self.filtered_issues = self
            .db
            .get_issues(project_id, self.issue_sort_order, issue_filter, assigned_to_filter)
            .unwrap_or_else(|e| {
                self.error_message = Some(format!("Failed to query issues: {}", e));
                Vec::new()
            });

        // Apply custom status ordering when sorting by status
        if matches!(
            self.issue_sort_order,
            IssueSortOrder::StatusAsc | IssueSortOrder::StatusDesc
        ) {
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

            self.filtered_issues.sort_by(|a, b| {
                let priority_a = status_priority(&a.status.name);
                let priority_b = status_priority(&b.status.name);
                if self.issue_sort_order == IssueSortOrder::StatusAsc {
                    priority_a.cmp(&priority_b)
                } else {
                    priority_b.cmp(&priority_a)
                }
            });
        }

        // Reset selection if out of bounds
        if self.issues_list_state >= self.filtered_issues.len() {
            self.issues_list_state = self.filtered_issues.len().saturating_sub(1);
        }
    }

    /// Rebuild the create issue form while preserving user-entered values
    /// This is called when the tracker changes to potentially show different fields
    pub fn rebuild_create_issue_form_preserving_values(&mut self) {
        // Extract all current values from the form
        let mut preserved_values: HashMap<String, FieldValue> = HashMap::new();
        let mut preserved_search_text: HashMap<String, String> = HashMap::new();
        let mut preserved_search_mode: HashMap<String, bool> = HashMap::new();
        let mut current_field_key: Option<String> = None;

        if let Some(form) = &self.create_issue_form {
            for field in &form.fields {
                if let Some(value) = form.get_value(&field.key) {
                    preserved_values.insert(field.key.clone(), value.clone());
                }
                // Also preserve search state
                let search_text = form.get_search_text(&field.key);
                if !search_text.is_empty() {
                    preserved_search_text.insert(field.key.clone(), search_text);
                }
                if form.is_search_mode(&field.key) {
                    preserved_search_mode.insert(field.key.clone(), true);
                }
            }
            // Remember which field was focused
            if let Some(field) = form.get_current_field() {
                current_field_key = Some(field.key.clone());
            }
        }

        // Get custom fields from cache for the new tracker
        let custom_fields = if let Some(tracker_id) = preserved_values.get("tracker_id").and_then(|v| v.as_option_id())
        {
            self.tracker_custom_fields_cache
                .get(&tracker_id)
                .cloned()
                .unwrap_or_else(Vec::new)
        } else {
            Vec::new()
        };

        // Create new form with cached custom fields (instant!)
        let mut new_form = IssueForm::new_issue_form_with_custom_fields(
            &self.trackers,
            &self.statuses,
            &self.priorities,
            &self.users,
            &self.categories,
            &custom_fields,
        );

        // Restore preserved values where they match field keys
        for (key, value) in preserved_values {
            // Check if this field still exists in the new form
            if new_form.fields.iter().any(|f| f.key == key) {
                new_form.set_value(key, value);
            }
        }

        // Restore search state for fields that still exist
        for (key, search_text) in preserved_search_text {
            if new_form.fields.iter().any(|f| f.key == key) {
                new_form.set_search_text(key, search_text);
            }
        }
        for (key, is_search_mode) in preserved_search_mode {
            if new_form.fields.iter().any(|f| f.key == key) {
                new_form.set_search_mode(key, is_search_mode);
            }
        }

        // Restore the current field index if the field still exists
        if let Some(key) = current_field_key {
            if let Some(idx) = new_form.fields.iter().position(|f| f.key == key) {
                new_form.current_field_idx = idx;
            } else {
                // Field doesn't exist anymore, default to tracker field (index 0)
                new_form.current_field_idx = 0;
            }
        }

        // Replace the form
        self.create_issue_form = Some(new_form);
    }

    /// Get the status name if cursor is on a status header (for grouped mode)
    pub fn get_status_at_cursor(&self) -> Option<String> {
        if !self.group_issues_by_status {
            return None;
        }

        // Build the display items to find what's at cursor
        let mut cursor_pos = 0;
        let mut status_order: Vec<String> = Vec::new();

        // Collect unique statuses in order of appearance
        for issue in &self.filtered_issues {
            if !status_order.contains(&issue.status.name) {
                status_order.push(issue.status.name.clone());
            }
        }

        for status_name in status_order {
            // Status header is at cursor_pos
            if cursor_pos == self.issues_list_state {
                return Some(status_name);
            }
            cursor_pos += 1;

            // Check if this status group is collapsed
            let is_collapsed = self.status_groups_collapsed.get(&status_name).copied().unwrap_or(false);
            if !is_collapsed {
                // Count issues in this status
                let issue_count = self
                    .filtered_issues
                    .iter()
                    .filter(|i| i.status.name == status_name)
                    .count();
                cursor_pos += issue_count;
            }
        }

        None
    }

    /// Get the status group that the cursor is currently within (header or any issue in the group)
    pub fn get_current_status_group(&self) -> Option<String> {
        if !self.group_issues_by_status {
            return None;
        }

        let mut cursor_pos = 0;
        let mut status_order: Vec<String> = Vec::new();

        // Collect unique statuses
        for issue in &self.filtered_issues {
            if !status_order.contains(&issue.status.name) {
                status_order.push(issue.status.name.clone());
            }
        }

        // Apply the SAME custom status ordering as UI rendering
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
        status_order.sort_by_key(|s| status_priority(s));

        for status_name in status_order {
            let header_pos = cursor_pos;
            cursor_pos += 1; // Move past header

            let is_collapsed = self.status_groups_collapsed.get(&status_name).copied().unwrap_or(false);

            if !is_collapsed {
                let issue_count = self
                    .filtered_issues
                    .iter()
                    .filter(|i| i.status.name == status_name)
                    .count();
                // Check if cursor is on header or any issue within this group
                if self.issues_list_state >= header_pos && self.issues_list_state < cursor_pos + issue_count {
                    return Some(status_name);
                }
                cursor_pos += issue_count;
            } else {
                // Cursor is on the collapsed header
                if self.issues_list_state == header_pos {
                    return Some(status_name);
                }
            }
        }

        None
    }

    /// Get the cursor position of a status group's header
    pub fn get_status_group_header_position(&self, target_status: &str) -> Option<usize> {
        if !self.group_issues_by_status {
            return None;
        }

        let mut cursor_pos = 0;
        let mut status_order: Vec<String> = Vec::new();

        // Collect unique statuses
        for issue in &self.filtered_issues {
            if !status_order.contains(&issue.status.name) {
                status_order.push(issue.status.name.clone());
            }
        }

        // Apply the same custom status ordering
        let status_priority = |status: &str| -> u32 {
            match status.to_lowercase().as_str() {
                s if s.contains("in progress") || s.contains("progress") => 1,
                s if s.contains("feedback") => 2,
                s if s.contains("new") => 3,
                s if s.contains("resolved") => 4,
                s if s.contains("closed") => 5,
                _ => 99,
            }
        };
        status_order.sort_by_key(|s| status_priority(s));

        for status_name in status_order {
            let header_pos = cursor_pos;

            if status_name == target_status {
                return Some(header_pos);
            }

            // Move past header
            cursor_pos += 1;

            // Move past issues if not collapsed
            let is_collapsed = self.status_groups_collapsed.get(&status_name).copied().unwrap_or(false);
            if !is_collapsed {
                let issue_count = self
                    .filtered_issues
                    .iter()
                    .filter(|i| i.status.name == status_name)
                    .count();
                cursor_pos += issue_count;
            }
        }

        None
    }

    /// Get the total number of visible items in grouped mode (headers + non-collapsed issues)
    pub fn get_visible_items_count(&self) -> usize {
        if !self.group_issues_by_status {
            return self.filtered_issues.len();
        }

        let mut count = 0;
        let mut status_order: Vec<String> = Vec::new();

        // Collect unique statuses
        for issue in &self.filtered_issues {
            if !status_order.contains(&issue.status.name) {
                status_order.push(issue.status.name.clone());
            }
        }

        // Apply the same custom status ordering
        let status_priority = |status: &str| -> u32 {
            match status.to_lowercase().as_str() {
                s if s.contains("in progress") || s.contains("progress") => 1,
                s if s.contains("feedback") => 2,
                s if s.contains("new") => 3,
                s if s.contains("resolved") => 4,
                s if s.contains("closed") => 5,
                _ => 99,
            }
        };
        status_order.sort_by_key(|s| status_priority(s));

        for status_name in status_order {
            // Add 1 for the header
            count += 1;

            let is_collapsed = self.status_groups_collapsed.get(&status_name).copied().unwrap_or(false);
            if !is_collapsed {
                // Add count of issues in this group
                let issue_count = self
                    .filtered_issues
                    .iter()
                    .filter(|i| i.status.name == status_name)
                    .count();
                count += issue_count;
            }
        }

        count
    }

    /// Get the issue if cursor is on an issue (not a header, for grouped mode)
    pub fn get_issue_at_cursor(&self) -> Option<&Issue> {
        if !self.group_issues_by_status {
            // Non-grouped mode: direct index
            return self.filtered_issues.get(self.issues_list_state);
        }

        // Grouped mode: need to calculate which issue
        let mut cursor_pos = 0;
        let mut status_order: Vec<String> = Vec::new();

        for issue in &self.filtered_issues {
            if !status_order.contains(&issue.status.name) {
                status_order.push(issue.status.name.clone());
            }
        }

        for status_name in status_order {
            // Skip status header
            cursor_pos += 1;

            let is_collapsed = self.status_groups_collapsed.get(&status_name).copied().unwrap_or(false);
            if !is_collapsed {
                // Look through issues in this status
                for issue in self.filtered_issues.iter().filter(|i| i.status.name == status_name) {
                    if cursor_pos == self.issues_list_state {
                        return Some(issue);
                    }
                    cursor_pos += 1;
                }
            }
        }

        None
    }

    /// Check if an issue has been updated since the last sync for its project
    pub fn is_issue_updated_since_last_sync(&self, issue: &Issue) -> bool {
        // Get the project's last sync time
        let project_sync = self
            .filtered_projects
            .iter()
            .find(|p| p.id == issue.project.id)
            .and_then(|p| p.last_issues_sync.as_ref());

        if let Some(last_sync) = project_sync {
            issue.updated_on > *last_sync
        } else {
            false
        }
    }

    /// Check if a project has any issues updated since last sync for that project
    pub fn has_project_updated_issues(&self, project: &Project) -> bool {
        if let Some(last_sync) = &project.last_issues_sync {
            if let Some(last_activity) = &project.last_issue_activity {
                last_activity > last_sync
            } else {
                false
            }
        } else {
            false
        }
    }
}
