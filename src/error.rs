/// Custom error types for the Redmine TUI application
///
/// Note: Currently using anyhow::Result throughout the codebase.
/// This module is kept for future migration to typed errors.
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum RedmineError {
    /// HTTP/Network errors from API calls
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Authentication/Authorization errors
    #[error("Authentication failed: {message}")]
    Auth { message: String },

    /// API errors with status codes
    #[error("API request failed with status {status}: {message}")]
    Api { status: u16, message: String },

    /// Validation errors (form inputs, etc.)
    #[error("Validation error: {0}")]
    Validation(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic errors
    #[error("{0}")]
    Other(String),
}

impl RedmineError {
    /// Create an API error from status and message
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Create a config error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create an auth error
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::Network(e) if e.is_timeout() => "Request timed out. Please check your connection.".to_string(),
            Self::Network(e) if e.is_connect() => "Cannot connect to server. Please check your network.".to_string(),
            Self::Auth { .. } => {
                "Authentication failed. Please check your API key in settings (press 'c').".to_string()
            }
            Self::Api { status: 404, .. } => "Resource not found.".to_string(),
            Self::Api { status: 403, .. } => {
                "Access denied. You may not have permission to perform this action.".to_string()
            }
            Self::Api { status: 422, message } => {
                format!("Validation failed: {}", message)
            }
            Self::Validation(msg) => msg.clone(),
            Self::Config(msg) => format!("Configuration error: {}", msg),
            Self::Database(_) => "Database error. Try refreshing data or clearing cache.".to_string(),
            _ => format!("{}", self),
        }
    }
}

// Allow conversion from anyhow::Error for compatibility
impl From<anyhow::Error> for RedmineError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = RedmineError::api(404, "Not found");
        assert!(matches!(err, RedmineError::Api { status: 404, .. }));

        let err = RedmineError::config("Invalid config");
        assert!(matches!(err, RedmineError::Config(_)));
    }

    #[test]
    fn test_user_messages() {
        let err = RedmineError::auth("Bad token");
        let msg = err.user_message();
        assert!(msg.contains("API key"));

        let err = RedmineError::api(404, "Not found");
        let msg = err.user_message();
        assert_eq!(msg, "Resource not found.");
    }
}
