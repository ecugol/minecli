mod app;
mod config;
mod db;
mod error;
mod events;
mod form_field;
mod issue_form;
mod redmine;
mod theme;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::EventHandler;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging()?;

    // Set up panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Try to restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new()?;

    // Load cached data from DB immediately
    app.apply_filters();

    // Load users from cache
    if let Err(e) = app.load_users_from_cache() {
        tracing::warn!("Failed to load users from cache: {}", e);
    }

    // Load initial data if configured (fetch from API in background)
    if app.client.is_some() {
        if let Err(e) = app.load_metadata().await {
            eprintln!("Warning: Failed to load metadata: {}", e);
            app.error_message = Some(format!("Failed to load metadata: {}", e));
        }
        // Only fetch from API if we have no cached data
        if app.total_projects == 0 {
            if let Err(e) = app.load_projects().await {
                eprintln!("Warning: Failed to load projects: {}", e);
                app.error_message = Some(format!("Failed to load projects: {}", e));
            }
        }

        // If users cache is empty, load them from API
        if app.users.is_empty() {
            if let Err(e) = app.load_all_users().await {
                tracing::warn!("Failed to load users from API: {}", e);
            }
        }
        
        // Clear status message after startup
        app.status_message = None;
    }

    // Event handler
    let event_handler = EventHandler::new();

    // Main loop
    let result = run_app(&mut terminal, &mut app, event_handler).await;

    // Always restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

/// Initialize logging to file
fn init_logging() -> Result<()> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    // Create logs directory
    let log_dir = directories::ProjectDirs::from("com", "minecli", "minecli")
        .map(|dirs| dirs.data_dir().join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));

    std::fs::create_dir_all(&log_dir)?;

    // File appender with daily rotation
    let file_appender = tracing_appender::rolling::daily(log_dir, "minecli.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Set up subscriber with env filter (default to warn level)
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("minecli=info,warn"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    tracing::info!("Redmine TUI started");
    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    event_handler: EventHandler,
) -> Result<()> {
    let mut last_selected_project: Option<u64> = None;
    let mut issue_to_load: Option<u64> = None;

    while app.running {
        // Clear status message if it's been shown for 3 seconds
        app.clear_expired_status_message(3);
        
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle refresh projects request
        if app.refresh_projects {
            app.refresh_projects = false;
            if let Err(e) = app.load_projects().await {
                app.error_message = Some(format!("Failed to load projects: {}", e));
                app.loading = false;
            }
        }

        // Handle refresh issues request
        if app.refresh_issues {
            app.refresh_issues = false;
            if let Err(e) = app.load_issues().await {
                app.error_message = Some(format!("Failed to load issues: {}", e));
                app.loading = false;
            }
        }
        
        // Handle incremental issue loading (one page per loop iteration)
        if app.issues_loading_in_progress {
            if let Err(e) = app.load_issues_next_page().await {
                app.error_message = Some(format!("Failed to load issues: {}", e));
                app.issues_loading_in_progress = false;
                app.loading = false;
            }
        }

        // Handle create new issue request
        if app.create_new_issue_flag {
            app.create_new_issue_flag = false;
            if let Err(e) = app.create_new_issue().await {
                app.error_message = Some(format!("Failed to create issue: {}", e));
            }
        }

        // Handle load users request
        if app.load_users_flag {
            app.load_users_flag = false;
            if let Some(project_id) = app.load_users_project_id.take() {
                app.status_message = Some("Loading project data...".to_string());
                if let Err(e) = app.load_project_users(project_id).await {
                    app.error_message = Some(format!("Failed to load users: {}", e));
                }
                if let Err(e) = app.load_project_metadata(project_id).await {
                    app.error_message = Some(format!("Failed to load project metadata: {}", e));
                }
                // Final status message set by load_project_users
            }
        }

        // Handle add comment request
        if app.add_comment_flag {
            app.add_comment_flag = false;
            if let Some(issue_id) = app.comment_issue_id.take() {
                if let Err(e) = app.add_comment_to_issue(issue_id).await {
                    app.error_message = Some(format!("Failed to add comment: {}", e));
                }
            }
        }

        // Handle bulk update execution
        if app.execute_bulk_update_flag {
            app.execute_bulk_update_flag = false;
            if let Err(e) = app.execute_bulk_update().await {
                app.error_message = Some(format!("Bulk update failed: {}", e));
            }
        }

        // Handle background loading of custom fields
        // Custom fields are loaded on-demand when creating issues, not at startup
        if app.load_custom_fields_flag {
            app.load_custom_fields_flag = false;
            // Skip for now - custom fields will be loaded lazily when needed
        }

        // Handle image loading
        if app.load_image_flag {
            app.load_image_flag = false;
            if let Some(url) = &app.viewing_image_url.clone() {
                if let Err(e) = app.download_single_image(url).await {
                    app.error_message = Some(format!("Failed to load image: {}", e));
                    app.show_image_viewer = false;
                    app.viewing_image_url = None;
                }
            }
        }

        // Handle project selection changes - load issues when a project is selected
        let current_project_id = app.selected_project.as_ref().map(|p| p.id);
        if current_project_id != last_selected_project && current_project_id.is_some() {
            // Load from DB first (instant)
            app.apply_filters();

            // Then fetch from API if we don't have recent data
            if app.total_issues == 0 {
                if let Err(e) = app.load_issues().await {
                    app.error_message = Some(format!("Failed to load issues: {}", e));
                    app.loading = false;
                }
            }
            last_selected_project = current_project_id;
        }

        // Handle issue detail loading
        if app.loading_issue && issue_to_load.is_none() {
            if let Some(issue) = &app.current_issue {
                issue_to_load = Some(issue.id);
            }
        }

        if let Some(issue_id) = issue_to_load {
            if let Err(e) = app.load_issue_detail(issue_id).await {
                app.error_message = Some(format!("Failed to load issue detail: {}", e));
                app.loading_issue = false;
            }
            issue_to_load = None;
        }

        // Handle events
        if let Ok(event) = event_handler.next() {
            match event {
                events::Event::Key(key) => {
                    app.handle_key(key);
                }
                events::Event::Mouse(mouse) => {
                    let size = terminal.size()?;
                    let rect = ratatui::layout::Rect {
                        x: 0,
                        y: 0,
                        width: size.width,
                        height: size.height,
                    };
                    app.handle_mouse(mouse, rect);
                }
                events::Event::Tick => {
                    // Handle periodic updates if needed
                }
            }
        }
    }

    Ok(())
}
