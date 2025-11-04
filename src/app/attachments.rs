use anyhow::Result;
use std::path::Path;

use super::state::App;
use crate::redmine::Upload;

impl App {
    /// Create a themed file explorer with help text
    pub fn create_file_explorer(&self) -> Result<ratatui_explorer::FileExplorer> {
        use ratatui::text::Line;
        use ratatui_explorer::Theme;
        
        // Create theme with title and help text
        let theme = Theme::default()
            .add_default_title()
            .with_title_bottom(|_| Line::from(" Enter: Select | h: Toggle Hidden | ESC: Cancel "));
        
        ratatui_explorer::FileExplorer::with_theme(theme)
            .map_err(|e| anyhow::anyhow!("Failed to create file explorer: {}", e))
    }
    
    /// Add a file to the pending attachments list
    pub fn add_attachment(&mut self, file_path: String) {
        if Path::new(&file_path).exists() {
            self.pending_attachments.push(file_path);
        }
    }
    
    /// Remove a file from the pending attachments list
    pub fn remove_attachment(&mut self, index: usize) {
        if index < self.pending_attachments.len() {
            self.pending_attachments.remove(index);
        }
    }
    
    /// Clear all pending attachments
    pub fn clear_attachments(&mut self) {
        self.pending_attachments.clear();
    }
    
    /// Get the list of pending attachments
    pub fn get_pending_attachments(&self) -> &[String] {
        &self.pending_attachments
    }
    
    /// Upload a file and return an Upload struct with the token
    async fn upload_file(&self, file_path: &str) -> Result<Upload> {
        let client = self.client.as_ref().ok_or_else(|| anyhow::anyhow!("No client"))?;
        
        // Read file contents
        let content = std::fs::read(file_path)?;
        
        // Get filename from path
        let filename = Path::new(file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
        
        // Upload the file
        let response = client.upload_file(filename, content).await?;
        
        // Create Upload struct
        Ok(Upload {
            token: response.upload.token,
            filename: Some(filename.to_string()),
            description: None,
            content_type: None,
        })
    }
    
    /// Upload multiple files and return a vector of Upload structs
    pub(super) async fn upload_files(&self, file_paths: &[String]) -> Result<Vec<Upload>> {
        let mut uploads = Vec::new();
        
        for file_path in file_paths {
            match self.upload_file(file_path).await {
                Ok(upload) => uploads.push(upload),
                Err(e) => {
                    tracing::warn!("Failed to upload file {}: {}", file_path, e);
                    return Err(e);
                }
            }
        }
        
        Ok(uploads)
    }
}
