// Form field handlers are in a separate module for better organization
#[path = "handler_modules/mod.rs"]
mod handler_modules;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::state::{App, InputMode, Pane, Screen};
use crate::form_field::FieldValue;
use crate::issue_form::IssueForm;
use crate::redmine::RedmineClient;

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.running = false;
            return;
        }

        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode_key(key),
            InputMode::Editing | InputMode::EditingUrl | InputMode::EditingApiKey => self.handle_editing_mode_key(key),
            InputMode::Searching => self.handle_search_mode_key(key),
            InputMode::CreatingIssue => self.handle_create_issue_key(key),
            InputMode::ReplyingToIssue => self.handle_reply_key(key),
            InputMode::BulkEditing => self.handle_bulk_editing_key(key),
            InputMode::AddingAttachment => self.handle_adding_attachment_key(key),
            InputMode::ManagingAttachments => self.handle_managing_attachments_key(key),
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, terminal_size: Rect) {
        // Don't handle mouse events if a popup is open
        if self.show_issue_popup 
            || self.show_create_issue_form 
            || self.bulk_edit_form.is_some()
            || self.update_issue_form.is_some()
            || self.show_image_viewer
            || self.show_error_popup
            || self.show_help_popup
        {
            return;
        }

        // Calculate pane boundaries (matching ui.rs layout)
        // Status bar takes 4 lines at bottom
        let main_height = terminal_size.height.saturating_sub(4);
        let projects_width = (terminal_size.width * 30) / 100;

        match mouse.kind {
            MouseEventKind::Down(_) | MouseEventKind::Up(_) => {
                let x = mouse.column;
                let y = mouse.row;

                // Check if click is in main area (not status bar)
                if y < main_height {
                    // Determine which pane was clicked
                    if x < projects_width {
                        // Projects pane clicked
                        self.focused_pane = Pane::Projects;
                        // Calculate which project was clicked (y - 1 for border)
                        if y >= 1 && !self.filtered_projects.is_empty() {
                            let clicked_index = (y - 1) as usize;
                            if clicked_index < self.filtered_projects.len() {
                                self.projects_list_state = clicked_index;
                                // Double click or Enter-like behavior: select project
                                if mouse.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                                    if let Some(project) = self.filtered_projects.get(clicked_index).cloned() {
                                        self.selected_project = Some(project);
                                        self.focused_pane = Pane::Issues;
                                        self.issues_list_state = 0;
                                    }
                                }
                            }
                        }
                    } else {
                        // Issues pane clicked - only allow if project is selected
                        if self.selected_project.is_some() {
                            self.focused_pane = Pane::Issues;
                            if y >= 1 && !self.filtered_issues.is_empty() {
                                let clicked_index = (y - 1) as usize;
                                if clicked_index < self.filtered_issues.len() {
                                    self.issues_list_state = clicked_index;
                                    // Click to open issue
                                    if mouse.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                                        if let Some(issue) = self.filtered_issues.get(clicked_index) {
                                            self.loading_issue = true;
                                            self.popup_scroll = 0;
                                            // Clone the issue directly instead of reconstructing field by field
                                            self.current_issue = Some(issue.clone());
                                            self.show_issue_popup = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                if self.show_issue_popup {
                    self.popup_scroll = self.popup_scroll.saturating_add(3);
                } else {
                    match self.focused_pane {
                        Pane::Projects => {
                            let max = self.filtered_projects.len().saturating_sub(1);
                            self.projects_list_state = (self.projects_list_state + 3).min(max);
                        }
                        Pane::Issues => {
                            let max = self.filtered_issues.len().saturating_sub(1);
                            self.issues_list_state = (self.issues_list_state + 3).min(max);
                        }
                    }
                }
            }
            MouseEventKind::ScrollUp => {
                if self.show_issue_popup {
                    self.popup_scroll = self.popup_scroll.saturating_sub(3);
                } else {
                    match self.focused_pane {
                        Pane::Projects => {
                            self.projects_list_state = self.projects_list_state.saturating_sub(3);
                        }
                        Pane::Issues => {
                            self.issues_list_state = self.issues_list_state.saturating_sub(3);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_normal_mode_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('?') => self.show_help_popup = !self.show_help_popup,
            KeyCode::Char('c') => self.screen = Screen::Config,
            KeyCode::Char('e') => {
                // Show error popup if there's an error
                if let Some(error) = &self.error_message {
                    self.show_error_popup = true;
                    // Also save error to a file for easier debugging
                    let error_file = "/tmp/minecli-last-error.txt";
                    if let Err(e) = std::fs::write(error_file, error) {
                        tracing::error!("Failed to write error file: {}", e);
                    } else {
                        self.status_message = Some(format!("Error saved to {}", error_file));
                    }
                }
            }
            KeyCode::Char('n') => {
                // Open new issue form (only if project is selected)
                if let Some(project) = &self.selected_project {
                    // Check if metadata is loaded
                    if self.trackers.is_empty() || self.statuses.is_empty() || self.priorities.is_empty() {
                        self.error_message = Some(format!(
                            "Metadata not loaded yet. Trackers: {}, Statuses: {}, Priorities: {}",
                            self.trackers.len(),
                            self.statuses.len(),
                            self.priorities.len()
                        ));
                        return;
                    }

                    let project_id = project.id;
                    // Trigger loading users for the project
                    self.load_users_flag = true;
                    self.load_users_project_id = Some(project_id);

                    // Get custom fields for default tracker from cache
                    let default_tracker_id = self
                        .trackers
                        .iter()
                        .find(|t| t.name.to_lowercase() == "task")
                        .or_else(|| self.trackers.first())
                        .map(|t| t.id);

                    let custom_fields = if let Some(tracker_id) = default_tracker_id {
                        self.tracker_custom_fields_cache
                            .get(&tracker_id)
                            .cloned()
                            .unwrap_or_else(Vec::new)
                    } else {
                        Vec::new()
                    };

                    // Create data-driven form with cached custom fields
                    self.create_issue_form = Some(IssueForm::new_issue_form_with_custom_fields(
                        &self.trackers,
                        &self.statuses,
                        &self.priorities,
                        &self.users,
                        &self.categories,
                        &custom_fields,
                    ));

                    self.show_create_issue_form = true;
                    self.input_mode = InputMode::CreatingIssue;
                } else {
                    self.error_message = Some("Please select a project first".to_string());
                }
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Searching;
                self.search_query.clear();
            }
            KeyCode::Esc => {
                if self.show_help_popup {
                    self.show_help_popup = false;
                } else if self.screen == Screen::Config {
                    // Check if configuration is complete before allowing exit
                    if self.config.redmine_url.is_empty() || self.config.api_key.is_empty() {
                        self.error_message = Some(
                            "Configuration incomplete! Please set both Redmine URL and API Key before continuing."
                                .to_string(),
                        );
                    } else {
                        self.screen = Screen::Main;
                        self.error_message = None;
                    }
                } else if self.show_error_popup {
                    self.show_error_popup = false;
                } else if self.show_image_viewer {
                    self.show_image_viewer = false;
                    self.viewing_image_url = None;
                    self.error_message = None;
                } else if self.show_issue_popup {
                    self.show_issue_popup = false;
                    self.current_issue = None;
                    self.popup_scroll = 0;
                    self.attachment_page = 0;
                    self.error_message = None;
                } else if self.show_create_issue_form {
                    self.show_create_issue_form = false;
                    self.input_mode = InputMode::Normal;
                    self.error_message = None;
                } else {
                    // Only clear filters if not closing a popup
                    self.project_filter.clear();
                    self.issue_filter.clear();
                    self.my_issues_filter = false;
                    self.apply_filters();
                    self.error_message = None;
                }
            }
            _ => match self.screen {
                Screen::Main => self.handle_main_screen_key(key),
                Screen::Config => self.handle_config_key(key),
                _ => {}
            },
        }
    }

    fn handle_search_mode_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                match self.focused_pane {
                    Pane::Projects => {
                        self.project_filter = self.search_query.clone();
                    }
                    Pane::Issues => {
                        self.issue_filter = self.search_query.clone();
                    }
                }
                self.apply_filters();
                self.search_query.clear();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
    }

    fn handle_editing_mode_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                // Validate and save based on which field is being edited
                match self.input_mode {
                    InputMode::EditingUrl => {
                        // Validate URL
                        let url = self.url_input.trim();
                        if url.is_empty() {
                            self.error_message = Some("URL cannot be empty".to_string());
                        } else if !url.starts_with("http://") && !url.starts_with("https://") {
                            self.error_message = Some("URL must start with http:// or https://".to_string());
                        } else if url.parse::<reqwest::Url>().is_err() {
                            self.error_message = Some("Invalid URL format".to_string());
                        } else {
                            // Valid URL
                            self.config.redmine_url = url.to_string();
                            self.input_mode = InputMode::Normal;
                            self.save_config();
                            self.status_message = Some("URL saved successfully".to_string());
                        }
                    }
                    InputMode::EditingApiKey => {
                        // Validate API key (must be exactly 40 chars)
                        let api_key = self.api_key_input.trim();
                        if api_key.is_empty() {
                            self.error_message = Some("API Key cannot be empty".to_string());
                        } else if api_key.len() != 40 {
                            self.error_message = Some(format!("API Key must be exactly 40 characters (current: {})", api_key.len()));
                        } else {
                            self.config.api_key = api_key.to_string();
                            self.input_mode = InputMode::Normal;
                            self.save_config();
                            self.status_message = Some("API Key saved successfully".to_string());
                        }
                    }
                    _ => {
                        self.input_mode = InputMode::Normal;
                    }
                }
            }
            KeyCode::Backspace => match self.input_mode {
                InputMode::EditingUrl => {
                    self.url_input.pop();
                }
                InputMode::EditingApiKey => {
                    self.api_key_input.pop();
                }
                _ => {}
            },
            KeyCode::Char(c) => match self.input_mode {
                InputMode::EditingUrl => {
                    self.url_input.push(c);
                }
                InputMode::EditingApiKey => {
                    self.api_key_input.push(c);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_main_screen_key(&mut self, key: KeyEvent) {
        if self.show_issue_popup {
            // Issue popup is open, handle scrolling and reply
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.popup_scroll = self.popup_scroll.saturating_add(1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.popup_scroll = self.popup_scroll.saturating_sub(1);
                }
                KeyCode::Char('G') => {
                    // Go to bottom (Shift+G) - scroll to the max calculated position
                    self.popup_scroll = self.popup_content_height.saturating_sub(1);
                }
                KeyCode::Char('g') => {
                    // Go to top
                    self.popup_scroll = 0;
                }
                KeyCode::Char('O') => {
                    // Open issue in browser (Shift+O)
                    if let Some(issue) = &self.current_issue {
                        let url = format!("{}/issues/{}", self.config.redmine_url, issue.id);
                        if let Err(e) = open::that(&url) {
                            self.error_message = Some(format!("Failed to open browser: {}", e));
                        } else {
                            self.status_message = Some(format!("Opening issue #{} in browser", issue.id));
                        }
                    }
                }
                KeyCode::Char('r') => {
                    // Open reply mode
                    if let Some(issue) = &self.current_issue {
                        // Load users and metadata for assignee dropdown if not already loaded
                        if self.users.is_empty() || self.categories.is_empty() {
                            self.load_users_flag = true;
                            self.load_users_project_id = Some(issue.project.id);
                        }

                        // Create data-driven update form with current values
                        self.update_issue_form = Some(IssueForm::update_issue_form(
                            &self.statuses,
                            &self.users,
                            &self.categories,
                            issue.status.id,
                            issue.assigned_to.as_ref().map(|a| a.id),
                            issue.done_ratio,
                            None, // category_id not tracked in issue struct yet
                        ));

                        self.input_mode = InputMode::ReplyingToIssue;
                    }
                }
                KeyCode::Char(c @ '1'..='9') => {
                    // View attachment - with pagination support
                    if let Some(issue) = &self.current_issue {
                        const ATTACHMENTS_PER_PAGE: usize = 9;
                        let page_index = (c as u8 - b'1') as usize;
                        let actual_index = self.attachment_page * ATTACHMENTS_PER_PAGE + page_index;

                        if let Some(attachment) = issue.attachments.get(actual_index) {
                            let url = if attachment.content_url.starts_with("http://")
                                || attachment.content_url.starts_with("https://")
                            {
                                attachment.content_url.clone()
                            } else {
                                format!("{}{}", self.config.redmine_url, attachment.content_url)
                            };

                            // Check if Shift is pressed - if so, open in browser regardless of type
                            let force_browser = key.modifiers.contains(KeyModifiers::SHIFT);

                            // If it's an image and Shift is NOT pressed, show in viewer
                            let is_image = attachment.content_type.as_ref().map_or(false, |ct| ct.starts_with("image/"));
                            if is_image && !force_browser {
                                self.show_image_viewer = true;
                                self.viewing_image_url = Some(url);
                                self.load_image_flag = true; // Trigger download in main loop
                                self.status_message = Some(format!("Loading image: {}", attachment.filename));
                            } else {
                                // Open in browser/default app (non-image OR Shift+Number pressed)
                                if let Err(e) = open::that(&url) {
                                    self.error_message = Some(format!("Failed to open attachment: {}", e));
                                } else {
                                    let action = if force_browser { "Opening in browser" } else { "Opening" };
                                    self.status_message = Some(format!("{}: {}", action, attachment.filename));
                                }
                            }
                        } else {
                            self.error_message = Some(format!("No attachment at position {}", page_index + 1));
                        }
                    }
                }
                KeyCode::Char('[') => {
                    // Previous page of attachments
                    if self.show_issue_popup && self.attachment_page > 0 {
                        self.attachment_page -= 1;
                        self.status_message = Some(format!("Page {}", self.attachment_page + 1));
                    }
                }
                KeyCode::Char(']') => {
                    // Next page of attachments
                    if self.show_issue_popup {
                        if let Some(issue) = &self.current_issue {
                            const ATTACHMENTS_PER_PAGE: usize = 9;
                            let total_pages = issue.attachments.len().div_ceil(ATTACHMENTS_PER_PAGE);
                            if self.attachment_page + 1 < total_pages {
                                self.attachment_page += 1;
                                self.status_message = Some(format!("Page {}", self.attachment_page + 1));
                            }
                        }
                    }
                }
                KeyCode::Char('J') => {
                    // Next issue (Shift+J)
                    if !self.filtered_issues.is_empty() {
                        let max_index = if self.group_issues_by_status {
                            self.get_visible_items_count().saturating_sub(1)
                        } else {
                            self.filtered_issues.len().saturating_sub(1)
                        };
                        
                        if self.issues_list_state < max_index {
                            self.issues_list_state += 1;
                            
                            // Load the new issue
                            if let Some(issue) = self.get_issue_at_cursor() {
                                let issue_clone = issue.clone();
                                self.loading_issue = true;
                                self.popup_scroll = 0;
                                self.attachment_page = 0;
                                self.current_issue = Some(issue_clone);
                                self.status_message = None; // Clear status to show help
                            }
                        }
                    }
                }
                KeyCode::Char('K') => {
                    // Previous issue (Shift+K)
                    if !self.filtered_issues.is_empty() && self.issues_list_state > 0 {
                        self.issues_list_state -= 1;
                        
                        // Load the new issue
                        if let Some(issue) = self.get_issue_at_cursor() {
                            let issue_clone = issue.clone();
                            self.loading_issue = true;
                            self.popup_scroll = 0;
                            self.attachment_page = 0;
                            self.current_issue = Some(issue_clone);
                            self.status_message = None; // Clear status to show help
                        }
                    }
                }
                _ => {}
            }
            return;
        }

        match key.code {
            // Pane navigation
            KeyCode::Char('h') => {
                self.focused_pane = Pane::Projects;
            }
            KeyCode::Char('l') => {
                // Only allow focusing Issues pane if a project is selected
                if self.selected_project.is_some() {
                    self.focused_pane = Pane::Issues;
                }
            }
            // Sort cycling - works regardless of focused pane if issues are loaded
            KeyCode::Char('s') => {
                if !self.filtered_issues.is_empty() {
                    self.issue_sort_order = self.issue_sort_order.next();
                    self.apply_filters(); // Re-query from DB with new sort order
                    self.issues_list_state = 0;
                    self.status_message = Some(format!("Sorted by: {}", self.issue_sort_order.as_str()));
                }
            }
            // Refresh data with Shift+P and Shift+I
            KeyCode::Char('P') => {
                self.refresh_projects = true;
                self.status_message = Some("Refreshing projects...".to_string());
            }
            KeyCode::Char('I') => {
                if self.selected_project.is_some() {
                    self.refresh_issues = true;
                    self.loading = true;
                    self.status_message = Some("Refreshing issues (this may take a moment for large projects)...".to_string());
                } else {
                    self.error_message = Some("No project selected".to_string());
                }
            }
            // Toggle "My Issues" filter
            KeyCode::Char('m') => {
                if self.current_user_id.is_some() {
                    self.my_issues_filter = !self.my_issues_filter;
                    self.apply_filters();
                    self.issues_list_state = 0;
                    if self.my_issues_filter {
                        self.status_message = Some("Showing only issues assigned to me".to_string());
                    } else {
                        self.status_message = Some("Showing all issues".to_string());
                    }
                } else {
                    self.error_message = Some("Loading user info...".to_string());
                }
            }
            // Toggle issues pane maximize
            KeyCode::Char('z') => {
                if self.selected_project.is_some() {
                    self.issues_pane_maximized = !self.issues_pane_maximized;
                    if self.issues_pane_maximized {
                        self.status_message = Some("Issues pane maximized (press 'z' to restore)".to_string());
                    } else {
                        self.status_message = Some("Split view restored".to_string());
                    }
                }
            }
            // Toggle bulk operation mode
            KeyCode::Char('b') | KeyCode::Char('B') => {
                if self.focused_pane == Pane::Issues && !self.filtered_issues.is_empty() {
                    self.toggle_bulk_mode();
                }
            }
            // Select all issues (in bulk mode)
            KeyCode::Char('a') if self.bulk_operation_mode => {
                self.select_all_issues();
            }
            // Clear all selections (in bulk mode)
            KeyCode::Char('A') if self.bulk_operation_mode => {
                self.deselect_all_issues();
            }
            // Show bulk edit form directly
            KeyCode::Char('x') if self.bulk_operation_mode => {
                self.show_bulk_edit_form();
            }
            // Toggle status grouping/folding
            KeyCode::Char('g') => {
                if !self.filtered_issues.is_empty() {
                    self.group_issues_by_status = !self.group_issues_by_status;
                    self.issues_list_state = 0;
                    if self.group_issues_by_status {
                        self.status_message = Some("Issues grouped by status (Space to collapse/expand)".to_string());
                    } else {
                        self.status_message = Some("Status grouping disabled".to_string());
                    }
                }
            }
            // List navigation
            KeyCode::Down | KeyCode::Char('j') => {
                match self.focused_pane {
                    Pane::Projects => {
                        let max = self.filtered_projects.len().saturating_sub(1);
                        if self.projects_list_state < max {
                            self.projects_list_state += 1;
                        }
                    }
                    Pane::Issues => {
                        // Calculate max based on visible items (respecting collapsed groups)
                        let max = if self.group_issues_by_status {
                            self.get_visible_items_count().saturating_sub(1)
                        } else {
                            self.filtered_issues.len().saturating_sub(1)
                        };
                        if self.issues_list_state < max {
                            self.issues_list_state += 1;
                        }
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => match self.focused_pane {
                Pane::Projects => {
                    if self.projects_list_state > 0 {
                        self.projects_list_state -= 1;
                    }
                }
                Pane::Issues => {
                    if self.issues_list_state > 0 {
                        self.issues_list_state -= 1;
                    }
                }
            },
            KeyCode::Char(' ') => {
                // Space key: toggle project/status group collapse OR toggle issue selection in bulk mode
                if self.focused_pane == Pane::Projects {
                    // Toggle project tree expansion
                    if let Some(project) = self.get_project_at_cursor().cloned() {
                        let project_id = project.id;
                        let is_collapsed = self.projects_collapsed.get(&project_id).copied().unwrap_or(false);
                        self.projects_collapsed.insert(project_id, !is_collapsed);

                        if is_collapsed {
                            self.status_message = Some(format!("Expanded: {}", project.name));
                        } else {
                            self.status_message = Some(format!("Collapsed: {}", project.name));
                        }
                    }
                } else if self.focused_pane == Pane::Issues {
                    if self.bulk_operation_mode {
                        // In bulk mode, space toggles issue selection
                        self.toggle_issue_selection();
                    } else if self.group_issues_by_status {
                        // Toggle status group collapse/expand in grouped mode
                        if let Some(status_name) = self.get_current_status_group() {
                            let is_collapsed = self.status_groups_collapsed.get(&status_name).copied().unwrap_or(false);
                            self.status_groups_collapsed.insert(status_name.clone(), !is_collapsed);

                            if is_collapsed {
                                self.status_message = Some(format!("Expanded: {}", status_name));
                            } else {
                                self.status_message = Some(format!("Collapsed: {}", status_name));
                            }
                        }
                    }
                }
            }
            KeyCode::Enter => {
                match self.focused_pane {
                    Pane::Projects => {
                        if let Some(project) = self.get_project_at_cursor().cloned() {
                            self.selected_project = Some(project);
                            self.focused_pane = Pane::Issues;
                            self.issues_list_state = 0;
                        }
                    }
                    Pane::Issues => {
                        // In grouped mode, Space toggles collapse - Enter still opens issue
                        if let Some(issue) = self.get_issue_at_cursor() {
                            // Clone the issue directly instead of reconstructing field by field
                            let issue_clone = issue.clone();

                            // Now mutate self
                            self.loading_issue = true;
                            self.popup_scroll = 0;
                            self.current_issue = Some(issue_clone);
                            self.show_issue_popup = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_config_key(&mut self, key: KeyEvent) {
        use crate::theme::{Theme, ThemeName};

        match key.code {
            KeyCode::Tab => {
                // Navigate forward through fields: URL -> API Key -> Theme -> Exclude Subprojects
                self.config_focused_field = (self.config_focused_field + 1) % 4;
            }
            KeyCode::BackTab => {
                // Navigate backward through fields
                self.config_focused_field = if self.config_focused_field == 0 {
                    3
                } else {
                    self.config_focused_field - 1
                };
            }
            KeyCode::Enter => {
                match self.config_focused_field {
                    0 => {
                        // Start editing URL
                        self.input_mode = InputMode::EditingUrl;
                        self.url_input = self.config.redmine_url.clone();
                    }
                    1 => {
                        // Start editing API Key
                        self.input_mode = InputMode::EditingApiKey;
                        self.api_key_input = self.config.api_key.clone();
                    }
                    2 => {
                        // Apply selected theme and save config
                        let themes = ThemeName::all();
                        if let Some(&selected_theme) = themes.get(self.theme_selector_index) {
                            self.config.theme = selected_theme;
                            self.theme = Theme::from_name(selected_theme);
                            self.status_message = Some(format!("Theme changed to: {}", selected_theme));
                        }
                        self.save_config();
                    }
                    3 => {
                        // Toggle exclude_subprojects checkbox
                        self.config.exclude_subprojects = !self.config.exclude_subprojects;
                        self.save_config();
                        let status = if self.config.exclude_subprojects {
                            "Subproject issues will be excluded"
                        } else {
                            "Subproject issues will be included"
                        };
                        self.status_message = Some(status.to_string());
                    }
                    _ => {}
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Only work when theme field is focused
                if self.config_focused_field == 2 && self.theme_selector_index > 0 {
                    self.theme_selector_index -= 1;
                    // Live preview: apply theme immediately
                    let themes = ThemeName::all();
                    if let Some(&selected_theme) = themes.get(self.theme_selector_index) {
                        self.config.theme = selected_theme;
                        self.theme = Theme::from_name(selected_theme);
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Only work when theme field is focused
                if self.config_focused_field == 2 {
                    let themes = ThemeName::all();
                    if self.theme_selector_index < themes.len() - 1 {
                        self.theme_selector_index += 1;
                        // Live preview: apply theme immediately
                        if let Some(&selected_theme) = themes.get(self.theme_selector_index) {
                            self.config.theme = selected_theme;
                            self.theme = Theme::from_name(selected_theme);
                        }
                    }
                }
            }
            KeyCode::Char(' ') => {
                // Toggle checkbox when on exclude_subprojects field
                if self.config_focused_field == 3 {
                    self.config.exclude_subprojects = !self.config.exclude_subprojects;
                    self.save_config();
                    let status = if self.config.exclude_subprojects {
                        "Subproject issues will be excluded"
                    } else {
                        "Subproject issues will be included"
                    };
                    self.status_message = Some(status.to_string());
                }
            }
            _ => {}
        }
    }

    fn handle_reply_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('s') => {
                    // Submit the update
                    if let Some(issue) = &self.current_issue {
                        self.add_comment_flag = true;
                        self.comment_issue_id = Some(issue.id);
                        self.input_mode = InputMode::Normal;
                        self.status_message = Some("Updating issue...".to_string());
                    }
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('@') => {
                // Open file explorer for attachment
                self.previous_input_mode = InputMode::ReplyingToIssue;
                self.input_mode = InputMode::AddingAttachment;
                
                match self.create_file_explorer() {
                    Ok(explorer) => {
                        self.file_explorer = Some(explorer);
                    }
                    Err(e) => self.error_message = Some(format!("Failed to open file explorer: {}", e)),
                }
                return;
            }
            KeyCode::Char('#') => {
                // Show attachment manager
                if !self.pending_attachments.is_empty() {
                    self.previous_input_mode = InputMode::ReplyingToIssue;
                    self.input_mode = InputMode::ManagingAttachments;
                    self.attachment_list_state = 0;
                }
                return;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.update_issue_form = None;
                self.clear_attachments();
            }
            KeyCode::Tab => {
                if let Some(form) = &mut self.update_issue_form {
                    // Check if current field is searchable dropdown in search mode
                    let is_in_search = form.get_current_field().is_some_and(|field| {
                        field.field_type == crate::form_field::FieldType::SearchableDropdown
                            && form.is_search_mode(&field.key)
                    });

                    if is_in_search {
                        // Let the searchable dropdown handler deal with it
                        self.handle_form_input(key, true);
                    } else {
                        form.next_field();
                    }
                }
            }
            KeyCode::BackTab => {
                if let Some(form) = &mut self.update_issue_form {
                    // Check if current field is searchable dropdown in search mode
                    let is_in_search = form.get_current_field().is_some_and(|field| {
                        field.field_type == crate::form_field::FieldType::SearchableDropdown
                            && form.is_search_mode(&field.key)
                    });

                    if is_in_search {
                        // Let the searchable dropdown handler deal with it
                        self.handle_form_input(key, true);
                    } else {
                        form.prev_field();
                    }
                }
            }
            _ => {
                self.handle_form_input(key, true);
            }
        }
    }

    fn handle_create_issue_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('s') => {
                    // Submit the form
                    self.submit_new_issue();
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('@') => {
                // Open file explorer for attachment
                self.previous_input_mode = InputMode::CreatingIssue;
                self.input_mode = InputMode::AddingAttachment;
                
                match self.create_file_explorer() {
                    Ok(explorer) => {
                        self.file_explorer = Some(explorer);
                    }
                    Err(e) => self.error_message = Some(format!("Failed to open file explorer: {}", e)),
                }
                return;
            }
            KeyCode::Char('#') => {
                // Show attachment manager
                if !self.pending_attachments.is_empty() {
                    self.previous_input_mode = InputMode::CreatingIssue;
                    self.input_mode = InputMode::ManagingAttachments;
                    self.attachment_list_state = 0;
                }
                return;
            }
            KeyCode::Esc => {
                self.show_create_issue_form = false;
                self.input_mode = InputMode::Normal;
                self.create_issue_form = None;
                self.clear_attachments();
            }
            KeyCode::Tab => {
                if let Some(form) = &mut self.create_issue_form {
                    // Check if current field is searchable dropdown in search mode
                    let is_in_search = form.get_current_field().is_some_and(|field| {
                        field.field_type == crate::form_field::FieldType::SearchableDropdown
                            && form.is_search_mode(&field.key)
                    });

                    if is_in_search {
                        // Let the searchable dropdown handler deal with it
                        self.handle_form_input(key, false);
                    } else {
                        form.next_field();
                    }
                }
            }
            KeyCode::BackTab => {
                if let Some(form) = &mut self.create_issue_form {
                    // Check if current field is searchable dropdown in search mode
                    let is_in_search = form.get_current_field().is_some_and(|field| {
                        field.field_type == crate::form_field::FieldType::SearchableDropdown
                            && form.is_search_mode(&field.key)
                    });

                    if is_in_search {
                        // Let the searchable dropdown handler deal with it
                        self.handle_form_input(key, false);
                    } else {
                        form.prev_field();
                    }
                }
            }
            _ => {
                self.handle_form_input(key, false);
            }
        }
    }

    fn submit_new_issue(&mut self) {
        if let Some(form) = &self.create_issue_form {
            // Validate required fields
            if let Err(e) = form.validate() {
                self.error_message = Some(e);
                return;
            }

            // Trigger creation in main loop
            self.create_new_issue_flag = true;
            self.status_message = Some("Creating issue...".to_string());
            self.show_create_issue_form = false;
            self.input_mode = InputMode::Normal;
        } else {
            self.error_message = Some("Form not initialized".to_string());
        }
    }

    fn save_config(&mut self) {
        if let Err(e) = self.config.save() {
            self.error_message = Some(format!("Failed to save config: {}", e));
        } else {
            // Only create client if both URL and API key are set
            if !self.config.redmine_url.is_empty() && !self.config.api_key.is_empty() {
                self.client = Some(RedmineClient::new(
                    self.config.redmine_url.clone(),
                    self.config.api_key.clone(),
                ));
            }
        }
    }

    fn handle_bulk_editing_key(&mut self, key: KeyEvent) {
        // Handle Ctrl+S to submit
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            self.execute_bulk_edit();
            return;
        }

        // Handle ESC to cancel
        if key.code == KeyCode::Esc {
            self.cancel_bulk_edit_form();
            return;
        }

        // Delegate to existing form field handlers (treating it like an update form)
        self.handle_bulk_form_input(key);
    }

    fn handle_bulk_form_input(&mut self, key: KeyEvent) {
        let form = self.bulk_edit_form.as_mut();

        // Extract field info before calling handler methods
        let field_info = form.and_then(|f| {
            f.get_current_field().map(|field| {
                (field.key.clone(), field.field_type.clone())
            })
        });

        if let Some((field_key, field_type)) = field_info {
            use crate::form_field::FieldType;

            match field_type {
                FieldType::Dropdown => {
                    self.handle_bulk_dropdown_input(key, &field_key);
                }
                FieldType::SearchableDropdown => {
                    self.handle_bulk_searchable_dropdown_input(key, &field_key);
                }
                _ => {}
            }
        }
    }

    fn handle_bulk_dropdown_input(&mut self, key: KeyEvent, field_key: &str) {
        if let Some(form) = &mut self.bulk_edit_form {
            match key.code {
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        form.prev_field();
                    } else {
                        form.next_field();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if let Some(field) = form.get_current_field() {
                        let current_value = form.get_value(field_key).and_then(|v| v.as_option_id());
                        if let Some(current_id) = current_value {
                            let current_idx = field.options.iter().position(|opt| opt.id == current_id).unwrap_or(0);
                            if current_idx > 0 {
                                let new_value = field.options[current_idx - 1].id;
                                form.set_value(field_key.to_string(), FieldValue::OptionId(Some(new_value)));
                            }
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if let Some(field) = form.get_current_field() {
                        let current_value = form.get_value(field_key).and_then(|v| v.as_option_id());
                        if let Some(current_id) = current_value {
                            let current_idx = field.options.iter().position(|opt| opt.id == current_id).unwrap_or(0);
                            if current_idx < field.options.len() - 1 {
                                let new_value = field.options[current_idx + 1].id;
                                form.set_value(field_key.to_string(), FieldValue::OptionId(Some(new_value)));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_bulk_searchable_dropdown_input(&mut self, key: KeyEvent, field_key: &str) {
        use crate::form_field::FieldValue;

        if let Some(form) = &mut self.bulk_edit_form {
            // Check if in search mode
            let is_search_mode = form.is_search_mode(field_key);

            match key.code {
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        form.prev_field();
                    } else {
                        form.next_field();
                    }
                }
                KeyCode::Char('/') if !is_search_mode => {
                    // Enter search mode
                    form.set_search_mode(field_key.to_string(), true);
                    form.set_search_text(field_key.to_string(), String::new());
                }
                KeyCode::Esc if is_search_mode => {
                    // Exit search mode
                    form.clear_search(field_key);
                }
                KeyCode::Char(c) if is_search_mode => {
                    // Append to search
                    let mut search = form.get_search_text(field_key);
                    search.push(c);
                    form.set_search_text(field_key.to_string(), search);
                }
                KeyCode::Backspace if is_search_mode => {
                    // Remove last char
                    let mut search = form.get_search_text(field_key);
                    search.pop();
                    form.set_search_text(field_key.to_string(), search);
                }
                KeyCode::Enter if is_search_mode => {
                    // Select first filtered option
                    let filtered = form.get_filtered_options(field_key);
                    if let Some(first_option) = filtered.first() {
                        form.set_value(field_key.to_string(), FieldValue::OptionId(Some(first_option.id)));
                        form.clear_search(field_key);
                    }
                }
                // Navigation works both in and out of search mode (like create form)
                KeyCode::Up if is_search_mode => {
                    self.navigate_bulk_dropdown(field_key, -1);
                }
                KeyCode::Down if is_search_mode => {
                    self.navigate_bulk_dropdown(field_key, 1);
                }
                KeyCode::Up | KeyCode::Char('k') if !is_search_mode => {
                    self.navigate_bulk_dropdown(field_key, -1);
                }
                KeyCode::Down | KeyCode::Char('j') if !is_search_mode => {
                    self.navigate_bulk_dropdown(field_key, 1);
                }
                _ => {}
            }
        }
    }

    fn navigate_bulk_dropdown(&mut self, field_key: &str, direction: i32) {
        use crate::form_field::FieldValue;

        if let Some(form) = &mut self.bulk_edit_form {
            let filtered = form.get_filtered_options(field_key);
            if filtered.is_empty() {
                return;
            }

            let current_value = form.get_value(field_key).and_then(|v| v.as_option_id());
            
            if let Some(current_id) = current_value {
                // Find current position in filtered list
                if let Some(current_idx) = filtered.iter().position(|opt| opt.id == current_id) {
                    let new_idx = if direction < 0 {
                        // Moving up
                        if current_idx > 0 {
                            current_idx - 1
                        } else {
                            filtered.len() - 1  // Wrap to bottom
                        }
                    } else {
                        // Moving down
                        if current_idx < filtered.len() - 1 {
                            current_idx + 1
                        } else {
                            0  // Wrap to top
                        }
                    };
                    form.set_value(field_key.to_string(), FieldValue::OptionId(Some(filtered[new_idx].id)));
                } else {
                    // Current value not in filtered list, select first
                    form.set_value(field_key.to_string(), FieldValue::OptionId(Some(filtered[0].id)));
                }
            } else {
                // No current value, select first or last based on direction
                let idx = if direction < 0 { filtered.len() - 1 } else { 0 };
                form.set_value(field_key.to_string(), FieldValue::OptionId(Some(filtered[idx].id)));
            }
        }
    }

    fn handle_adding_attachment_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                // Get the selected file
                if let Some(explorer) = &self.file_explorer {
                    let file = explorer.current();
                    let file_path = file.path();
                    if file_path.is_file() {
                        let path = file_path.to_string_lossy().to_string();
                        let filename = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        self.add_attachment(path);
                        self.status_message = Some(format!("Added {} ({} total)", 
                            filename,
                            self.pending_attachments.len()));
                        self.file_explorer = None;
                        self.input_mode = self.previous_input_mode;
                        return;
                    }
                }
                // If it's a directory or no file selected, forward the event
                if let Some(explorer) = &mut self.file_explorer {
                    let _ = explorer.handle(&crossterm::event::Event::Key(key));
                }
            }
            KeyCode::Esc => {
                // Cancel
                self.file_explorer = None;
                self.input_mode = self.previous_input_mode;
                self.status_message = None;
            }
            _ => {
                // Forward other keys to the explorer
                if let Some(explorer) = &mut self.file_explorer {
                    let _ = explorer.handle(&crossterm::event::Event::Key(key));
                }
            }
        }
    }

    fn handle_managing_attachments_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Close attachment manager
                self.input_mode = self.previous_input_mode;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.attachment_list_state > 0 {
                    self.attachment_list_state -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.attachment_list_state < self.pending_attachments.len().saturating_sub(1) {
                    self.attachment_list_state += 1;
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                // Delete selected attachment
                if !self.pending_attachments.is_empty() && self.attachment_list_state < self.pending_attachments.len() {
                    self.remove_attachment(self.attachment_list_state);
                    if self.attachment_list_state >= self.pending_attachments.len() && self.attachment_list_state > 0 {
                        self.attachment_list_state -= 1;
                    }
                    self.status_message = Some(format!("Removed attachment ({} remaining)", self.pending_attachments.len()));
                    
                    // Close manager if no attachments left
                    if self.pending_attachments.is_empty() {
                        self.input_mode = self.previous_input_mode;
                    }
                }
            }
            _ => {}
        }
    }
}
