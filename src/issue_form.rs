use crate::form_field::{FieldOption, FieldValue, FormField};
use crate::redmine::{IssueCategory, IssueCustomField, IssueStatus, Priority, Tracker, User};
use std::collections::HashMap;

/// Manages the state of an issue form (create or update)
pub struct IssueForm {
    pub fields: Vec<FormField>,
    pub values: HashMap<String, FieldValue>,
    pub current_field_idx: usize,
    pub search_text: HashMap<String, String>, // For searchable dropdowns
    pub search_mode: HashMap<String, bool>,   // Track if field is in search mode
    pub scroll_offset: usize,                 // For scrollable forms
}

impl IssueForm {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            values: HashMap::new(),
            current_field_idx: 0,
            search_text: HashMap::new(),
            search_mode: HashMap::new(),
            scroll_offset: 0,
        }
    }

    /// Build a form for creating a new issue
    pub fn new_issue_form(
        trackers: &[Tracker],
        statuses: &[IssueStatus],
        priorities: &[Priority],
        users: &[User],
        categories: &[IssueCategory],
    ) -> Self {
        Self::new_issue_form_with_custom_fields(trackers, statuses, priorities, users, categories, &[])
    }

    /// Build a form for creating a new issue with custom fields
    pub fn new_issue_form_with_custom_fields(
        trackers: &[Tracker],
        statuses: &[IssueStatus],
        priorities: &[Priority],
        users: &[User],
        categories: &[IssueCategory],
        custom_fields: &[IssueCustomField],
    ) -> Self {
        let mut form = Self::new();

        // Tracker - required dropdown (default to "Task" if exists) - FIRST FIELD
        let tracker_options: Vec<FieldOption> = trackers
            .iter()
            .map(|t| FieldOption {
                id: t.id,
                name: t.name.clone(),
            })
            .collect();
        let default_tracker_id = trackers
            .iter()
            .find(|t| t.name.to_lowercase() == "task")
            .or_else(|| trackers.first())
            .map(|t| t.id);
        let mut tracker_field = FormField::new_searchable_dropdown("tracker_id", "Tracker", tracker_options, true);
        tracker_field.default_value = Some(FieldValue::OptionId(default_tracker_id));
        form.add_field(tracker_field);

        // Subject - required text field
        form.add_field(FormField::new_text("subject", "Subject", true));

        // Description - optional textarea
        form.add_field(FormField::new_textarea("description", "Description", false));

        // Status - required dropdown (default to "New" if exists)
        let status_options: Vec<FieldOption> = statuses
            .iter()
            .map(|s| FieldOption {
                id: s.id,
                name: s.name.clone(),
            })
            .collect();
        let default_status_id = statuses
            .iter()
            .find(|s| s.name.to_lowercase() == "new")
            .or_else(|| statuses.first())
            .map(|s| s.id);
        let mut status_field = FormField::new_searchable_dropdown("status_id", "Status", status_options, true);
        status_field.default_value = Some(FieldValue::OptionId(default_status_id));
        form.add_field(status_field);

        // Priority - required dropdown (default to "Normal" if exists)
        let priority_options: Vec<FieldOption> = priorities
            .iter()
            .map(|p| FieldOption {
                id: p.id,
                name: p.name.clone(),
            })
            .collect();
        let default_priority_id = priorities
            .iter()
            .find(|p| p.name.to_lowercase() == "normal")
            .or_else(|| priorities.first())
            .map(|p| p.id);
        let mut priority_field = FormField::new_searchable_dropdown("priority_id", "Priority", priority_options, true);
        priority_field.default_value = Some(FieldValue::OptionId(default_priority_id));
        form.add_field(priority_field);

        // Assignee - optional searchable dropdown, sorted alphabetically
        let mut user_options: Vec<FieldOption> = users
            .iter()
            .filter(|u| u.id != 0) // Exclude id=0, we'll add it manually
            .map(|u| FieldOption {
                id: u.id,
                name: format!("{} {}", u.firstname, u.lastname).trim().to_string(),
            })
            .collect();
        
        // Sort alphabetically by name (case-insensitive)
        user_options.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        // Add "(Unassigned)" at the beginning
        user_options.insert(0, FieldOption {
            id: 0,
            name: "(Unassigned)".to_string(),
        });
        
        form.add_field(FormField::new_searchable_dropdown(
            "assigned_to_id",
            "Assignee",
            user_options,
            false,
        ));

        // Category - optional dropdown (project-specific)
        if !categories.is_empty() {
            let category_options: Vec<FieldOption> = std::iter::once(FieldOption {
                id: 0,
                name: "(None)".to_string(),
            })
            .chain(categories.iter().map(|c| FieldOption {
                id: c.id,
                name: c.name.clone(),
            }))
            .collect();
            form.add_field(FormField::new_dropdown(
                "category_id",
                "Category",
                category_options,
                false,
            ));
        }

        // Start Date - optional date field
        form.add_field(FormField::new_date("start_date", "Start Date", false));

        // Due Date - optional date field
        form.add_field(FormField::new_date("due_date", "Due Date", false));

        // Estimated Hours - optional float field
        form.add_field(FormField::new_float("estimated_hours", "Estimated Hours", false));

        // Done Ratio - optional progress field
        form.add_field(FormField::new_progress("done_ratio", "% Done"));

        // Add custom fields (tracker-specific)
        // Note: Custom fields are currently treated as text fields since we only
        // have access to IssueCustomField (value only) not CustomField (with format metadata).
        // To properly support typed custom fields, we would need to:
        // 1. Change tracker_custom_fields_cache to store CustomField instead of IssueCustomField
        // 2. Load and cache CustomField data from project details API
        // 3. Match on field_format to create appropriate field types
        for custom_field in custom_fields {
            let field_key = format!("custom_field_{}", custom_field.id);
            form.add_field(FormField::new_text(
                &field_key,
                &custom_field.name,
                false, // Not required - we don't have metadata to determine this
            ));
        }

        form
    }

    /// Build a form for bulk editing multiple issues
    pub fn bulk_edit_form(
        statuses: &[IssueStatus],
        priorities: &[Priority],
        users: &[User],
    ) -> Self {
        let mut form = Self::new();

        // Status - SEARCHABLE dropdown
        let status_options: Vec<FieldOption> = statuses
            .iter()
            .map(|s| FieldOption {
                id: s.id,
                name: s.name.clone(),
            })
            .collect();
        let default_status_id = statuses.first().map(|s| s.id);
        let mut status_field = FormField::new_searchable_dropdown("status_id", "Status", status_options, false);
        status_field.default_value = Some(FieldValue::OptionId(default_status_id));
        form.add_field(status_field);

        // Priority - SEARCHABLE dropdown
        let priority_options: Vec<FieldOption> = priorities
            .iter()
            .map(|p| FieldOption {
                id: p.id,
                name: p.name.clone(),
            })
            .collect();
        let default_priority_id = priorities.first().map(|p| p.id);
        let mut priority_field = FormField::new_searchable_dropdown("priority_id", "Priority", priority_options, false);
        priority_field.default_value = Some(FieldValue::OptionId(default_priority_id));
        form.add_field(priority_field);

        // Assignee - SEARCHABLE dropdown with "(Unassigned)" option, sorted alphabetically
        let mut user_options: Vec<FieldOption> = users
            .iter()
            .filter(|u| u.id != 0) // Exclude any existing id=0 user
            .map(|u| FieldOption {
                id: u.id,
                name: format!("{} {}", u.firstname, u.lastname).trim().to_string(),
            })
            .collect();
        
        // Sort alphabetically by name
        user_options.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        // Add "(Unassigned)" at the beginning
        user_options.insert(0, FieldOption {
            id: 0,
            name: "(Unassigned)".to_string(),
        });
        
        let mut assignee_field = FormField::new_searchable_dropdown("assigned_to_id", "Assignee", user_options, false);
        assignee_field.default_value = Some(FieldValue::OptionId(Some(0))); // Default to unassigned
        form.add_field(assignee_field);

        form
    }

    /// Build a form for replying to/updating an issue
    pub fn update_issue_form(
        statuses: &[IssueStatus],
        users: &[User],
        categories: &[IssueCategory],
        current_status_id: u64,
        current_assignee_id: Option<u64>,
        current_done_ratio: Option<u32>,
        current_category_id: Option<u64>,
    ) -> Self {
        let mut form = Self::new();

        // Notes/Comment - optional textarea
        form.add_field(FormField::new_textarea("notes", "Notes (comment)", false));

        // Status - optional searchable dropdown
        let status_options: Vec<FieldOption> = statuses
            .iter()
            .map(|s| FieldOption {
                id: s.id,
                name: s.name.clone(),
            })
            .collect();
        let mut status_field = FormField::new_searchable_dropdown("status_id", "Status", status_options, false);
        status_field.default_value = Some(FieldValue::OptionId(Some(current_status_id)));
        form.add_field(status_field);

        // Assignee - optional searchable dropdown, sorted alphabetically
        let mut user_options: Vec<FieldOption> = users
            .iter()
            .filter(|u| u.id != 0) // Exclude id=0, we'll add it manually
            .map(|u| FieldOption {
                id: u.id,
                name: format!("{} {}", u.firstname, u.lastname).trim().to_string(),
            })
            .collect();
        
        // Sort alphabetically by name (case-insensitive)
        user_options.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        
        // Add "(Unassigned)" at the beginning
        user_options.insert(0, FieldOption {
            id: 0,
            name: "(Unassigned)".to_string(),
        });
        
        let mut assignee_field = FormField::new_searchable_dropdown("assigned_to_id", "Assignee", user_options, false);
        assignee_field.default_value = Some(FieldValue::OptionId(current_assignee_id));
        form.add_field(assignee_field);

        // Progress/Done Ratio - optional progress field
        let mut progress_field = FormField::new_progress("done_ratio", "% Done");
        progress_field.default_value = Some(FieldValue::Number(current_done_ratio));
        form.add_field(progress_field);

        // Category - optional searchable dropdown (project-specific)
        if !categories.is_empty() {
            let category_options: Vec<FieldOption> = std::iter::once(FieldOption {
                id: 0,
                name: "(None)".to_string(),
            })
            .chain(categories.iter().map(|c| FieldOption {
                id: c.id,
                name: c.name.clone(),
            }))
            .collect();
            let mut category_field = FormField::new_searchable_dropdown("category_id", "Category", category_options, false);
            category_field.default_value = Some(FieldValue::OptionId(current_category_id));
            form.add_field(category_field);
        }

        // Due Date - optional date field
        form.add_field(FormField::new_date("due_date", "Due Date", false));

        // Estimated Hours - optional float field
        form.add_field(FormField::new_float("estimated_hours", "Estimated Hours", false));

        // Private Notes - optional checkbox
        form.add_field(FormField::new_checkbox("private_notes", "Private Notes"));

        form
    }

    pub fn add_field(&mut self, field: FormField) {
        // Initialize value with default
        if let Some(default_value) = &field.default_value {
            self.values.insert(field.key.clone(), default_value.clone());
        }
        self.fields.push(field);
    }

    pub fn get_current_field(&self) -> Option<&FormField> {
        self.fields.get(self.current_field_idx)
    }

    pub fn get_current_field_mut(&mut self) -> Option<&mut FormField> {
        self.fields.get_mut(self.current_field_idx)
    }

    pub fn next_field(&mut self) {
        if self.current_field_idx < self.fields.len().saturating_sub(1) {
            self.current_field_idx += 1;
        } else {
            self.current_field_idx = 0;
        }
    }

    pub fn prev_field(&mut self) {
        if self.current_field_idx > 0 {
            self.current_field_idx -= 1;
        } else {
            self.current_field_idx = self.fields.len().saturating_sub(1);
        }
    }

    /// Update scroll offset to ensure current field is visible
    /// visible_fields: number of fields that can be displayed at once
    pub fn update_scroll(&mut self, visible_fields: usize) {
        if visible_fields == 0 {
            return;
        }

        // If current field is above visible area, scroll up
        if self.current_field_idx < self.scroll_offset {
            self.scroll_offset = self.current_field_idx;
        }
        // If current field is below visible area, scroll down
        else if self.current_field_idx >= self.scroll_offset + visible_fields {
            self.scroll_offset = self.current_field_idx.saturating_sub(visible_fields - 1);
        }
    }

    pub fn get_value(&self, key: &str) -> Option<&FieldValue> {
        self.values.get(key)
    }

    pub fn set_value(&mut self, key: String, value: FieldValue) {
        self.values.insert(key, value);
    }

    pub fn get_search_text(&self, key: &str) -> String {
        self.search_text.get(key).cloned().unwrap_or_else(String::new)
    }

    pub fn set_search_text(&mut self, key: String, text: String) {
        self.search_text.insert(key, text);
    }

    pub fn is_search_mode(&self, key: &str) -> bool {
        self.search_mode.get(key).copied().unwrap_or(false)
    }

    pub fn set_search_mode(&mut self, key: String, mode: bool) {
        self.search_mode.insert(key, mode);
    }

    pub fn clear_search(&mut self, key: &str) {
        self.search_text.remove(key);
        self.search_mode.remove(key);
    }

    /// Validate required fields
    pub fn validate(&self) -> Result<(), String> {
        for field in &self.fields {
            if field.required {
                let value = self.values.get(&field.key);
                match value {
                    Some(FieldValue::Text(s)) if s.is_empty() => {
                        return Err(format!("{} is required", field.label));
                    }
                    Some(FieldValue::OptionId(None)) => {
                        return Err(format!("{} is required", field.label));
                    }
                    None => {
                        return Err(format!("{} is required", field.label));
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Get filtered options for a searchable dropdown
    /// Prioritizes exact matches over partial matches
    pub fn get_filtered_options(&self, field_key: &str) -> Vec<&FieldOption> {
        if let Some(field) = self.fields.iter().find(|f| f.key == field_key) {
            let search = self.get_search_text(field_key).to_lowercase();
            if search.is_empty() {
                field.options.iter().collect()
            } else {
                // First, check for exact match
                let exact_matches: Vec<&FieldOption> = field
                    .options
                    .iter()
                    .filter(|opt| opt.name.to_lowercase() == search)
                    .collect();

                // If we have an exact match, return only that
                if !exact_matches.is_empty() {
                    exact_matches
                } else {
                    // Otherwise, return all partial matches
                    field
                        .options
                        .iter()
                        .filter(|opt| opt.name.to_lowercase().contains(&search))
                        .collect()
                }
            }
        } else {
            Vec::new()
        }
    }
}

impl Default for IssueForm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::form_field::{FieldValue, FormField};

    #[test]
    fn test_form_validation_required_fields() {
        let mut form = IssueForm::new();

        // Add required field
        let mut field = FormField::new_text("subject", "Subject", true);
        field.default_value = None;
        form.add_field(field);

        // Should fail - required field empty
        assert!(form.validate().is_err());

        // Set value
        form.set_value("subject".to_string(), FieldValue::Text("Test".to_string()));

        // Should pass now
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_form_validation_optional_fields() {
        let mut form = IssueForm::new();

        // Add optional field
        form.add_field(FormField::new_text("description", "Description", false));

        // Should pass even when empty
        assert!(form.validate().is_ok());

        // Should still pass with value
        form.set_value("description".to_string(), FieldValue::Text("Test desc".to_string()));
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_form_navigation() {
        let mut form = IssueForm::new();
        form.add_field(FormField::new_text("field1", "Field 1", false));
        form.add_field(FormField::new_text("field2", "Field 2", false));
        form.add_field(FormField::new_text("field3", "Field 3", false));

        assert_eq!(form.current_field_idx, 0);

        form.next_field();
        assert_eq!(form.current_field_idx, 1);

        form.next_field();
        assert_eq!(form.current_field_idx, 2);

        form.next_field(); // Should wrap to 0
        assert_eq!(form.current_field_idx, 0);

        form.prev_field(); // Should wrap to last
        assert_eq!(form.current_field_idx, 2);
    }

    #[test]
    fn test_searchable_dropdown_filtering() {
        let mut form = IssueForm::new();
        let options = vec![
            FieldOption {
                id: 1,
                name: "Alice".to_string(),
            },
            FieldOption {
                id: 2,
                name: "Bob".to_string(),
            },
            FieldOption {
                id: 3,
                name: "Charlie".to_string(),
            },
            FieldOption {
                id: 4,
                name: "Alice Johnson".to_string(),
            },
        ];

        form.add_field(FormField::new_searchable_dropdown("user", "User", options, false));

        // Empty search returns all
        let filtered = form.get_filtered_options("user");
        assert_eq!(filtered.len(), 4);

        // Partial match
        form.set_search_text("user".to_string(), "ali".to_string());
        let filtered = form.get_filtered_options("user");
        assert_eq!(filtered.len(), 2); // Alice and Alice Johnson

        // Exact match
        form.set_search_text("user".to_string(), "alice".to_string());
        let filtered = form.get_filtered_options("user");
        assert_eq!(filtered.len(), 1); // Only "Alice" exact match
        assert_eq!(filtered[0].name, "Alice");
    }

    #[test]
    fn test_form_value_storage() {
        let mut form = IssueForm::new();
        let mut field = FormField::new_text("key1", "Key 1", false);
        field.default_value = None; // Explicitly no default
        form.add_field(field);

        // Should have no value initially (no default set)
        let initial_value = form.get_value("key1");
        assert!(initial_value.is_none() || matches!(initial_value, Some(FieldValue::Text(s)) if s.is_empty()));

        form.set_value("key1".to_string(), FieldValue::Text("value1".to_string()));
        assert!(form.get_value("key1").is_some());
        assert_eq!(form.get_value("key1").unwrap().as_text().unwrap(), "value1");
    }
}
