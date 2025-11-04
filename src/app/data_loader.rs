use anyhow::Result;
use std::collections::HashMap;

use super::state::App;
use crate::issue_form::IssueForm;
use crate::redmine::User;

impl App {
    /// Load global metadata (trackers, statuses, priorities, current user)
    pub async fn load_metadata(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            self.status_message = Some("Loading metadata...".to_string());
            
            // Load trackers
            if let Ok(response) = client.get_trackers().await {
                self.trackers = response.trackers;
            }

            // Load statuses
            if let Ok(response) = client.get_issue_statuses().await {
                self.statuses = response.issue_statuses;
            }

            // Load priorities
            if let Ok(response) = client.get_priorities().await {
                self.priorities = response.issue_priorities;
            }

            // Load current user
            if let Ok(response) = client.get_current_user().await {
                self.current_user_id = Some(response.user.id);
            }
            
            self.status_message = Some("Metadata loaded successfully".to_string());
        }
        Ok(())
    }

    /// Load issues for the currently selected project (incrementally)
    pub async fn load_issues(&mut self) -> Result<()> {
        // Initialize loading state
        if !self.issues_loading_in_progress {
            self.issues_loading_in_progress = true;
            self.issues_loaded_count = 0;
            self.issues_total_count = 0;
            self.issues_temp_buffer.clear();
            self.loading = true;
        }
        
        Ok(())
    }
    
    /// Load next page of issues (called from event loop)
    pub async fn load_issues_next_page(&mut self) -> Result<bool> {
        if let Some(client) = &self.client {
            if let Some(project) = &self.selected_project {
                let project_name = project.name.clone();
                let project_id = project.id;
                let limit = 100;
                let offset = self.issues_loaded_count as u32;
                let exclude_subprojects = self.config.exclude_subprojects;

                match client.get_issues(Some(project_id), Some("*"), limit, offset, exclude_subprojects).await {
                    Ok(response) => {
                        let total_count = response.total_count.unwrap_or(0);
                        let received_count = response.issues.len();
                        
                        self.issues_temp_buffer.extend(response.issues);
                        self.issues_loaded_count = self.issues_temp_buffer.len();
                        self.issues_total_count = total_count as usize;
                        
                        // Update progress message
                        self.status_message = Some(format!(
                            "Loading issues... {}/{}", 
                            self.issues_loaded_count, 
                            self.issues_total_count
                        ));

                        // Check if we've loaded all issues
                        if received_count < limit as usize || self.issues_loaded_count >= self.issues_total_count {
                            // All pages loaded - finalize
                            if let Err(e) = self.db.insert_issues(&self.issues_temp_buffer) {
                                self.error_message = Some(format!("Failed to store issues: {}", e));
                            }

                            self.apply_filters();
                            self.status_message = Some(format!("Loaded {} issues from {}", self.issues_loaded_count, project_name));
                            
                            self.issues_loading_in_progress = false;
                            self.issues_temp_buffer.clear();
                            self.loading = false;
                            return Ok(true); // Done
                        }
                        
                        return Ok(false); // More pages to load
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load issues: {}", e));
                        self.issues_loading_in_progress = false;
                        self.issues_temp_buffer.clear();
                        self.loading = false;
                        return Ok(true); // Stop on error
                    }
                }
            }
        }
        Ok(true) // No client or project, stop
    }

    pub async fn load_projects(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            self.loading = true;
            self.status_message = Some("Loading projects...".to_string());
            let mut all_projects = Vec::new();
            let mut had_error = false;

            // Fetch all projects in batches
            let limit = 100;
            let mut offset = 0;
            loop {
                match client.get_projects(limit, offset).await {
                    Ok(response) => {
                        let fetched = response.projects.len();
                        all_projects.extend(response.projects);

                        // Check if we've fetched all projects
                        if let Some(total) = response.total_count {
                            if all_projects.len() >= total as usize {
                                break;
                            }
                        }

                        // If we got less than limit, we're done
                        if fetched < limit as usize {
                            break;
                        }

                        offset += limit;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load projects: {}", e));
                        had_error = true;
                        self.loading = false;
                        break;
                    }
                }
            }

            // Only proceed with storage and filtering if we didn't have an error
            if !had_error {
                // Store in database
                if let Err(e) = self.db.insert_projects(&all_projects) {
                    self.error_message = Some(format!("Failed to store projects: {}", e));
                }

                // Update last issue activity for projects by fetching 100 most recent issues
                self.status_message = Some("Updating project activity dates...".to_string());
                
                // Fetch 100 most recently updated issues across all projects
                match client.get_recent_issues(100, 0).await {
                    Ok(response) => {
                        let issues = response.issues;
                        
                        // Store the fetched issues in the database cache
                        if let Err(e) = self.db.insert_issues(&issues) {
                            tracing::warn!("Failed to cache recent issues: {}", e);
                        }
                        
                        // Use a HashMap to track the most recent issue per project
                        let mut project_last_activity: HashMap<u64, chrono::DateTime<chrono::Utc>> = HashMap::new();
                        
                        // Update project activity from the fetched issues
                        for issue in issues {
                            let project_id = issue.project.id;
                            // Only keep the most recent update per project
                            project_last_activity.entry(project_id)
                                .and_modify(|existing| {
                                    if issue.updated_on > *existing {
                                        *existing = issue.updated_on;
                                    }
                                })
                                .or_insert(issue.updated_on);
                        }
                        
                        // Update the database with the collected activity dates
                        for (project_id, last_activity) in project_last_activity {
                            if let Err(e) = self.db.update_project_last_activity(project_id, last_activity) {
                                tracing::warn!("Failed to update last activity for project {}: {}", project_id, e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch recent issues for project activity update: {}", e);
                    }
                }

                // Load all users for the system (useful for journal display)
                if let Err(e) = self.load_all_users().await {
                    tracing::warn!("Failed to load users: {}", e);
                    // Don't show error to user, it's not critical
                }

                self.apply_filters();
                self.projects_list_state = 0;
                self.status_message = Some(format!("Loaded {} projects", all_projects.len()));
                self.loading = false;
            }
        }
        Ok(())
    }

    /// Load all users from the Redmine instance
    pub async fn load_all_users(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            self.status_message = Some("Loading users...".to_string());
            let mut all_users = Vec::new();
            let limit = 100;
            let mut offset = 0;

            loop {
                match client.get_users(limit, offset).await {
                    Ok(response) => {
                        let fetched = response.users.len();
                        all_users.extend(response.users);

                        if let Some(total) = response.total_count {
                            if all_users.len() >= total as usize {
                                break;
                            }
                        }

                        if fetched < limit as usize {
                            break;
                        }

                        offset += limit;
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to fetch users: {}", e));
                    }
                }
            }

            // Save users to database
            if let Err(e) = self.db.insert_users(&all_users) {
                tracing::warn!("Failed to cache users in DB: {}", e);
            }

            self.users = all_users.clone();
            self.status_message = Some(format!("Loaded {} users", all_users.len()));
            tracing::info!("Loaded {} users", self.users.len());
        }
        Ok(())
    }

    pub fn load_users_from_cache(&mut self) -> Result<()> {
        self.users = self.db.get_users()?;
        tracing::info!("Loaded {} users from cache", self.users.len());
        Ok(())
    }

    pub async fn load_project_metadata(&mut self, project_id: u64) -> Result<()> {
        if let Some(client) = &self.client {
            if let Ok(response) = client.get_project_detail(project_id).await {
                self.categories = response.project.issue_categories;
                // Use project-specific trackers instead of global ones
                if !response.project.trackers.is_empty() {
                    self.trackers = response.project.trackers;
                }
            }
        }
        Ok(())
    }

    pub async fn load_project_users(&mut self, project_id: u64) -> Result<()> {
        if let Some(client) = &self.client {
            if let Ok(response) = client.get_project_memberships(project_id).await {
                // Extract users from memberships
                self.users = response
                    .memberships
                    .into_iter()
                    .filter_map(|m| {
                        m.user.map(|u| User {
                            id: u.id,
                            login: String::new(), // Not provided in membership
                            firstname: u.name.clone(),
                            lastname: String::new(),
                            mail: None,
                        })
                    })
                    .collect();
                // Add "None" option at the beginning
                self.users.insert(
                    0,
                    User {
                        id: 0,
                        login: String::from("none"),
                        firstname: String::from("(None)"),
                        lastname: String::new(),
                        mail: None,
                    },
                );

                // Rebuild create form if it exists (to update user list)
                if self.show_create_issue_form {
                    self.create_issue_form = Some(IssueForm::new_issue_form(
                        &self.trackers,
                        &self.statuses,
                        &self.priorities,
                        &self.users,
                        &self.categories,
                    ));
                }
                
                self.status_message = Some(format!("Project data loaded ({} users)", self.users.len()));
            }
        }
        Ok(())
    }

    pub async fn load_issue_detail(&mut self, issue_id: u64) -> Result<()> {
        if let Some(client) = &self.client {
            self.loading_issue = true;
            self.status_message = Some(format!("Loading issue #{}...", issue_id));
            match client.get_issue(issue_id).await {
                Ok(response) => {
                    let issue = response.issue;

                    // Store in database
                    if let Err(e) = self.db.insert_issue_with_journals(&issue) {
                        self.error_message = Some(format!("Failed to store issue: {}", e));
                    }

                    self.current_issue = Some(issue.clone());
                    self.status_message = Some(format!("Loaded issue #{}: {}", issue.id, issue.subject));
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load issue: {}", e));
                }
            }
            self.loading_issue = false;
        }
        Ok(())
    }

    pub async fn add_comment_to_issue(&mut self, issue_id: u64) -> Result<()> {
        let client = match &self.client {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        if let Some(form) = &self.update_issue_form {
            // Extract values from form
            let notes = form
                .get_value("notes")
                .and_then(|v| v.as_text())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            let status_id = form.get_value("status_id").and_then(|v| v.as_option_id());

            let assigned_to_id = form.get_value("assigned_to_id").and_then(|v| v.as_option_id());

            let done_ratio = form.get_value("done_ratio").and_then(|v| v.as_number());

            let category_id = form.get_value("category_id").and_then(|v| v.as_option_id());

            let due_date = form
                .get_value("due_date")
                .and_then(|v| v.as_text())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            let estimated_hours = form.get_value("estimated_hours").and_then(|v| v.as_float());

            let private_notes = form.get_value("private_notes").map(|v| v.as_bool());

            // Upload any pending attachments
            let uploads = if !self.pending_attachments.is_empty() {
                self.status_message = Some("Uploading attachments...".to_string());
                match self.upload_files(&self.pending_attachments.clone()).await {
                    Ok(uploads) => {
                        self.pending_attachments.clear();
                        Some(uploads)
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to upload attachments: {}", e));
                        return Ok(());
                    }
                }
            } else {
                None
            };

            let update = crate::redmine::UpdateIssue {
                subject: None,
                description: None,
                status_id,
                priority_id: None,
                assigned_to_id,
                done_ratio,
                category_id,
                start_date: None,
                due_date,
                estimated_hours,
                notes,
                private_notes,
                uploads,
            };

            match client.update_issue_with_comment(issue_id, update).await {
                Ok(()) => {
                    self.status_message = Some("Issue updated successfully".to_string());
                    // Reload the issue detail to show the changes
                    self.load_issue_detail(issue_id).await?;

                    // Refresh the issues list to reflect the new state
                    let project_id = self.selected_project.as_ref().map(|p| p.id);
                    if let Some(pid) = project_id {
                        let exclude_subprojects = self.config.exclude_subprojects;
                        if let Ok(response) = client.get_issues(Some(pid), Some("*"), 100, 0, exclude_subprojects).await {
                            // Update the issue in database
                            if let Err(e) = self.db.insert_issues(&response.issues) {
                                self.error_message = Some(format!("Failed to update issues cache: {}", e));
                            }
                            // Refresh the filtered view
                            self.apply_filters();
                        }
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to update issue: {}", e));
                }
            }
        }
        Ok(())
    }

    pub async fn create_new_issue(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            if let Some(project) = &self.selected_project {
                if let Some(form) = &self.create_issue_form {
                    // Extract values from form
                    let subject = form
                        .get_value("subject")
                        .and_then(|v| v.as_text())
                        .unwrap_or("")
                        .to_string();

                    let description = form
                        .get_value("description")
                        .and_then(|v| v.as_text())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    let tracker_id = form
                        .get_value("tracker_id")
                        .and_then(|v| v.as_option_id())
                        .ok_or_else(|| anyhow::anyhow!("Tracker is required"))?;

                    let status_id = form
                        .get_value("status_id")
                        .and_then(|v| v.as_option_id())
                        .ok_or_else(|| anyhow::anyhow!("Status is required"))?;

                    let priority_id = form
                        .get_value("priority_id")
                        .and_then(|v| v.as_option_id())
                        .ok_or_else(|| anyhow::anyhow!("Priority is required"))?;

                    let assigned_to_id = form.get_value("assigned_to_id").and_then(|v| v.as_option_id());

                    let category_id = form.get_value("category_id").and_then(|v| v.as_option_id());

                    let start_date = form
                        .get_value("start_date")
                        .and_then(|v| v.as_text())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    let due_date = form
                        .get_value("due_date")
                        .and_then(|v| v.as_text())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    let estimated_hours = form.get_value("estimated_hours").and_then(|v| v.as_float());

                    let done_ratio = form.get_value("done_ratio").and_then(|v| v.as_number());

                    // Upload any pending attachments
                    let uploads = if !self.pending_attachments.is_empty() {
                        self.status_message = Some("Uploading attachments...".to_string());
                        match self.upload_files(&self.pending_attachments.clone()).await {
                            Ok(uploads) => {
                                self.pending_attachments.clear();
                                Some(uploads)
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Failed to upload attachments: {}", e));
                                return Ok(());
                            }
                        }
                    } else {
                        None
                    };

                    let new_issue = crate::redmine::CreateIssue {
                        project_id: project.id,
                        tracker_id,
                        status_id,
                        priority_id,
                        subject,
                        description,
                        assigned_to_id,
                        category_id,
                        start_date,
                        due_date,
                        estimated_hours,
                        done_ratio,
                        uploads,
                    };

                    match client.create_issue(new_issue).await {
                        Ok(response) => {
                            // Store in database
                            if let Err(e) = self.db.insert_issue_with_journals(&response.issue) {
                                self.error_message = Some(format!("Failed to store issue: {}", e));
                            }
                            self.status_message = Some(format!("Created issue #{}", response.issue.id));
                            // Refresh the list
                            self.apply_filters();
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to create issue: {}", e));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Download and cache a single image by URL
    pub async fn download_single_image(&mut self, url: &str) -> Result<()> {
        // Skip if already cached
        if self.attachment_images.contains_key(url) {
            return Ok(());
        }

        let client = match &self.client {
            Some(c) => c.clone(),
            None => return Ok(()),
        };

        // Download the image
        match client.download_attachment(url).await {
            Ok(bytes) => {
                // Try to decode the image
                match image::load_from_memory(&bytes) {
                    Ok(dynamic_image) => {
                        // Store original dimensions
                        let (width, height) = (dynamic_image.width(), dynamic_image.height());
                        self.image_dimensions.insert(url.to_string(), (width, height));

                        // Create a protocol for this image using the picker
                        let protocol = self.image_picker.new_resize_protocol(dynamic_image);
                        self.attachment_images.insert(url.to_string(), protocol);
                        self.status_message = Some("Image loaded successfully".to_string());
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to decode image: {}", e);
                    }
                }
            }
            Err(e) => {
                anyhow::bail!("Failed to download image: {}", e);
            }
        }

        Ok(())
    }
}
