use anyhow::Result;
use chrono::{DateTime, Utc};
use ratatui_image::picker::Picker;
use std::collections::{HashMap, HashSet};

use crate::config::Config;
use crate::db::Database;
use crate::issue_form::IssueForm;
use crate::redmine::{
    Issue, IssueCategory, IssueCustomField, IssueStatus, Priority, Project, RedmineClient, Tracker, User,
};
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Main, // Two-pane view with projects and issues
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pane {
    Projects,
    Issues,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    #[allow(dead_code)]
    Editing,
    EditingUrl,
    EditingApiKey,
    Searching,
    CreatingIssue,
    ReplyingToIssue,
    BulkEditing,
    AddingAttachment,
    ManagingAttachments,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IssueSortOrder {
    UpdatedDesc,  // Most recently updated first (default)
    StatusAsc,    // Status alphabetically A-Z
    StatusDesc,   // Status alphabetically Z-A
    PriorityAsc,  // Priority low to high
    PriorityDesc, // Priority high to low
}

impl IssueSortOrder {
    pub fn next(&self) -> Self {
        match self {
            IssueSortOrder::UpdatedDesc => IssueSortOrder::StatusAsc,
            IssueSortOrder::StatusAsc => IssueSortOrder::StatusDesc,
            IssueSortOrder::StatusDesc => IssueSortOrder::PriorityAsc,
            IssueSortOrder::PriorityAsc => IssueSortOrder::PriorityDesc,
            IssueSortOrder::PriorityDesc => IssueSortOrder::UpdatedDesc,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            IssueSortOrder::UpdatedDesc => "Recent",
            IssueSortOrder::StatusAsc => "Status ↑",
            IssueSortOrder::StatusDesc => "Status ↓",
            IssueSortOrder::PriorityAsc => "Priority ↑",
            IssueSortOrder::PriorityDesc => "Priority ↓",
        }
    }
}

pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub input_mode: InputMode,
    pub config: Config,
    pub theme: Theme,
    pub client: Option<RedmineClient>,
    pub db: Database,

    // Data (now loaded from DB)
    pub filtered_projects: Vec<Project>,
    pub total_projects: usize,
    pub filtered_issues: Vec<Issue>,
    pub total_issues: usize,
    pub current_issue: Option<Issue>,
    pub trackers: Vec<Tracker>,
    pub statuses: Vec<IssueStatus>,
    pub priorities: Vec<Priority>,
    pub users: Vec<User>,
    pub categories: Vec<IssueCategory>,
    pub tracker_custom_fields: Vec<IssueCustomField>, // Custom fields for selected tracker
    pub tracker_custom_fields_cache: HashMap<u64, Vec<IssueCustomField>>, // Cache: tracker_id -> custom fields

    // UI State
    pub focused_pane: Pane,
    pub projects_list_state: usize,
    pub issues_list_state: usize,
    pub selected_project: Option<Project>,
    pub issues_pane_maximized: bool,
    pub show_issue_popup: bool,
    pub popup_scroll: usize,
    pub popup_content_height: usize,
    pub show_image_viewer: bool,
    pub viewing_image_url: Option<String>,
    pub show_error_popup: bool,
    pub show_help_popup: bool,  // Show help popup
    pub attachment_page: usize, // Current page of attachments (0-based)
    pub load_image_flag: bool,  // Flag to trigger image download
    pub loading: bool,
    pub loading_issue: bool,
    pub status_message: Option<String>,
    pub status_message_time: Option<std::time::Instant>,
    pub error_message: Option<String>,
    pub refresh_projects: bool,
    pub refresh_issues: bool,
    pub create_new_issue_flag: bool,
    pub load_users_flag: bool,
    pub load_users_project_id: Option<u64>,
    pub add_comment_flag: bool,
    pub comment_issue_id: Option<u64>,
    pub load_custom_fields_flag: bool, // Background loading of custom fields

    // Incremental issue loading state
    pub issues_loading_in_progress: bool,
    pub issues_loaded_count: usize,
    pub issues_total_count: usize,
    pub issues_temp_buffer: Vec<crate::redmine::Issue>,

    // Search/Filter
    pub search_query: String,
    pub project_filter: String,
    pub issue_filter: String,
    pub issue_sort_order: IssueSortOrder,
    pub my_issues_filter: bool,       // Filter for issues assigned to me
    pub current_user_id: Option<u64>, // Current user ID

    // Status grouping/folding
    pub status_groups_collapsed: HashMap<String, bool>, // Track collapsed status groups
    pub group_issues_by_status: bool,                   // Enable/disable status grouping

    // Project tree folding
    pub projects_collapsed: HashMap<u64, bool>, // Track collapsed projects (project_id -> collapsed)

    // Bulk operations
    pub selected_issues: HashSet<u64>,           // IDs of selected issues for bulk operations
    pub bulk_operation_mode: bool,               // Whether bulk selection mode is active
    pub show_bulk_action_menu: bool,             // Show bulk action menu popup (deprecated)
    pub bulk_action_menu_state: usize,           // Selected action in bulk menu (deprecated)
    pub bulk_edit_form: Option<IssueForm>,       // Bulk edit form (reuses IssueForm infrastructure)
    pub execute_bulk_update_flag: bool,          // Flag to trigger bulk update in main loop

    // Input fields
    pub api_key_input: String,
    pub url_input: String,
    pub attachment_input: String,

    // Theme selection
    pub theme_selector_index: usize, // Selected theme in config screen
    pub config_focused_field: usize, // 0=URL, 1=API Key, 2=Theme, 3=Exclude Subprojects

    // Issue creation form (data-driven)
    pub show_create_issue_form: bool,
    pub create_issue_form: Option<IssueForm>,
    pub update_issue_form: Option<IssueForm>,

    // File attachments for create/update
    pub pending_attachments: Vec<String>, // File paths to upload
    pub previous_input_mode: InputMode, // To return to after adding attachment
    pub attachment_list_state: usize, // Selected index in attachment list
    pub file_explorer: Option<ratatui_explorer::FileExplorer>, // File explorer widget

    // Last sync timestamps
    pub last_projects_sync: Option<DateTime<Utc>>,

    // Image rendering
    pub image_picker: Picker,
    pub attachment_images: HashMap<String, ratatui_image::protocol::StatefulProtocol>, // URL -> image protocol
    pub image_dimensions: HashMap<String, (u32, u32)>, // URL -> (width, height) in pixels
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load().unwrap_or_default();
        let theme = Theme::from_name(config.theme);

        // Find index of current theme for selector
        let theme_selector_index = crate::theme::ThemeName::all()
            .iter()
            .position(|&t| t == config.theme)
            .unwrap_or(0);

        let client = if config.is_configured() {
            Some(RedmineClient::new(config.redmine_url.clone(), config.api_key.clone()))
        } else {
            None
        };

        // Initialize database
        let db_path = directories::ProjectDirs::from("com", "minecli", "minecli")
            .map(|dirs| dirs.data_dir().join("cache.db"))
            .unwrap_or_else(|| std::path::PathBuf::from("redmine-cache.db"));
        let db = Database::new(db_path)?;

        // Initialize image picker - use from_query_stdio() to detect terminal capabilities
        // Fallback to a default font size if detection fails
        let image_picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::from_fontsize((8, 16)));

        Ok(Self {
            running: true,
            screen: if config.is_configured() {
                Screen::Main
            } else {
                Screen::Config
            },
            input_mode: InputMode::Normal,
            config,
            theme,
            client,
            db,
            filtered_projects: Vec::new(),
            total_projects: 0,
            filtered_issues: Vec::new(),
            total_issues: 0,
            current_issue: None,
            trackers: Vec::new(),
            statuses: Vec::new(),
            priorities: Vec::new(),
            users: Vec::new(),
            categories: Vec::new(),
            tracker_custom_fields: Vec::new(),
            tracker_custom_fields_cache: HashMap::new(),
            focused_pane: Pane::Projects,
            projects_list_state: 0,
            issues_list_state: 0,
            selected_project: None,
            issues_pane_maximized: false,
            show_issue_popup: false,
            popup_scroll: 0,
            popup_content_height: 0,
            show_image_viewer: false,
            viewing_image_url: None,
            show_error_popup: false,
            show_help_popup: false,
            attachment_page: 0,
            load_image_flag: false,
            loading: false,
            loading_issue: false,
            status_message: None,
            status_message_time: None,
            error_message: None,
            refresh_projects: false,
            refresh_issues: false,
            create_new_issue_flag: false,
            load_users_flag: false,
            load_users_project_id: None,
            add_comment_flag: false,
            comment_issue_id: None,
            load_custom_fields_flag: false,
            search_query: String::new(),
            project_filter: String::new(),
            issue_filter: String::new(),
            issue_sort_order: IssueSortOrder::UpdatedDesc,
            my_issues_filter: false,
            current_user_id: None,
            status_groups_collapsed: HashMap::new(),
            group_issues_by_status: false, // Disabled by default
            projects_collapsed: HashMap::new(),
            selected_issues: HashSet::new(),
            bulk_operation_mode: false,
            show_bulk_action_menu: false,
            bulk_action_menu_state: 0,
            bulk_edit_form: None,
            execute_bulk_update_flag: false,
            api_key_input: String::new(),
            url_input: String::new(),
            attachment_input: String::new(),
            theme_selector_index,
            config_focused_field: 1, // Start focused on API Key (most important)
            show_create_issue_form: false,
            create_issue_form: None,
            update_issue_form: None,
            pending_attachments: Vec::new(),
            previous_input_mode: InputMode::Normal,
            attachment_list_state: 0,
            file_explorer: None,
            last_projects_sync: None,
            image_picker,
            attachment_images: HashMap::new(),
            image_dimensions: HashMap::new(),
            issues_loading_in_progress: false,
            issues_loaded_count: 0,
            issues_total_count: 0,
            issues_temp_buffer: Vec::new(),
        })
    }
    
    /// Set status message with timestamp for auto-clear
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
        self.status_message_time = Some(std::time::Instant::now());
    }
    
    /// Clear status message if it's been shown for more than the given duration
    pub fn clear_expired_status_message(&mut self, duration_secs: u64) {
        // If status message exists but no timestamp, set it now (for backwards compatibility)
        if self.status_message.is_some() && self.status_message_time.is_none() {
            self.status_message_time = Some(std::time::Instant::now());
        }
        
        if let Some(time) = self.status_message_time {
            if time.elapsed().as_secs() >= duration_secs {
                self.status_message = None;
                self.status_message_time = None;
            }
        }
    }
}
