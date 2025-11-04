//! Bulk operations for performing actions on multiple issues at once.
use super::state::App;

impl App {
    /// Toggle bulk operation mode on/off
    pub fn toggle_bulk_mode(&mut self) {
        self.bulk_operation_mode = !self.bulk_operation_mode;
        if !self.bulk_operation_mode {
            // Clear selections when exiting bulk mode
            self.selected_issues.clear();
        }
        self.status_message = Some(if self.bulk_operation_mode {
            "Bulk mode enabled - Space to select, B to perform actions".to_string()
        } else {
            "Bulk mode disabled".to_string()
        });
    }

    /// Toggle selection of current issue
    pub fn toggle_issue_selection(&mut self) {
        if !self.bulk_operation_mode {
            return;
        }

        if let Some(issue) = self.filtered_issues.get(self.issues_list_state) {
            let issue_id = issue.id;
            if self.selected_issues.contains(&issue_id) {
                self.selected_issues.remove(&issue_id);
            } else {
                self.selected_issues.insert(issue_id);
            }
            self.status_message = Some(format!("{} issues selected", self.selected_issues.len()));
        }
    }

    /// Select all visible issues
    pub fn select_all_issues(&mut self) {
        if !self.bulk_operation_mode {
            return;
        }

        for issue in &self.filtered_issues {
            self.selected_issues.insert(issue.id);
        }
        self.status_message = Some(format!("Selected all {} issues", self.selected_issues.len()));
    }

    /// Deselect all issues
    pub fn deselect_all_issues(&mut self) {
        self.selected_issues.clear();
        self.status_message = Some("Cleared all selections".to_string());
    }



    /// Get count of selected issues
    pub fn get_selected_count(&self) -> usize {
        self.selected_issues.len()
    }

    /// Check if an issue is selected
    pub fn is_issue_selected(&self, issue_id: u64) -> bool {
        self.selected_issues.contains(&issue_id)
    }



    /// Show the bulk edit form (called directly when pressing 'x' in bulk mode)
    pub fn show_bulk_edit_form(&mut self) {
        if self.selected_issues.is_empty() {
            self.error_message = Some("No issues selected".to_string());
            return;
        }

        // Verify we have the required data
        if self.statuses.is_empty() && self.priorities.is_empty() {
            self.error_message = Some("Required data not loaded. Please refresh.".to_string());
            return;
        }

        // Load users if needed (for current project)
        if self.users.is_empty() {
            if let Some(project) = &self.selected_project {
                self.load_users_flag = true;
                self.load_users_project_id = Some(project.id);
                self.status_message = Some("Loading users...".to_string());
                // Will show form after users are loaded
                return;
            }
        }

        // Create the bulk edit form
        let form = crate::issue_form::IssueForm::bulk_edit_form(&self.statuses, &self.priorities, &self.users);
        self.bulk_edit_form = Some(form);
        self.input_mode = crate::app::state::InputMode::BulkEditing;
    }

    /// Execute the bulk update with all form values
    pub fn execute_bulk_edit(&mut self) {
        // Trigger the update
        self.execute_bulk_update_flag = true;

        // Close form
        self.cancel_bulk_edit_form();
    }

    /// Cancel bulk edit form
    pub fn cancel_bulk_edit_form(&mut self) {
        self.bulk_edit_form = None;
        self.input_mode = crate::app::state::InputMode::Normal;
    }

    /// Execute bulk update on all selected issues
    pub async fn execute_bulk_update(&mut self) -> anyhow::Result<()> {
        use crate::redmine::UpdateIssue;

        if self.selected_issues.is_empty() {
            return Ok(());
        }

        let client = self.client.as_ref().ok_or_else(|| anyhow::anyhow!("No API client"))?;

        // Extract values from the form
        let status_id = self.bulk_edit_form.as_ref()
            .and_then(|form| form.get_value("status_id"))
            .and_then(|v| v.as_option_id());

        let priority_id = self.bulk_edit_form.as_ref()
            .and_then(|form| form.get_value("priority_id"))
            .and_then(|v| v.as_option_id());

        let assigned_to_id = self.bulk_edit_form.as_ref()
            .and_then(|form| form.get_value("assigned_to_id"))
            .and_then(|v| v.as_option_id())
            .map(|id| if id == 0 { None } else { Some(id) }); // 0 = unassigned

        // Build the update payload
        let update = UpdateIssue {
            subject: None,
            description: None,
            status_id,
            priority_id,
            assigned_to_id: assigned_to_id.flatten(),
            done_ratio: None,
            category_id: None,
            start_date: None,
            due_date: None,
            estimated_hours: None,
            notes: None,
            private_notes: None,
            uploads: None,
        };

        // Track success/failure counts
        let total = self.selected_issues.len();
        let mut success_count = 0;
        let mut failed_issues = Vec::new();

        // Update each issue
        for issue_id in &self.selected_issues {
            match client.update_issue(*issue_id, update.clone()).await {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    failed_issues.push((*issue_id, e.to_string()));
                }
            }
        }

        // Exit bulk mode and clear selections
        self.bulk_operation_mode = false;
        self.selected_issues.clear();

        // Show result message
        if failed_issues.is_empty() {
            self.status_message = Some(format!("Successfully updated {} issue(s)", success_count));
        } else {
            let error_details = failed_issues
                .iter()
                .map(|(id, err)| format!("  #{}: {}", id, err))
                .collect::<Vec<_>>()
                .join("\n");
            self.error_message = Some(format!(
                "Updated {}/{} issues. Failed issues:\n{}",
                success_count, total, error_details
            ));
        }

        // Refresh issues from API
        self.refresh_issues = true;

        Ok(())
    }


}
