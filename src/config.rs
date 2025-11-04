use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::theme::ThemeName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub redmine_url: String,
    pub api_key: String,
    #[serde(default)]
    pub theme: ThemeName,
    #[serde(default)]
    pub exclude_subprojects: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            redmine_url: String::new(),
            api_key: String::new(),
            theme: ThemeName::default(),
            exclude_subprojects: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        confy::load("minecli", "config").context("Failed to load configuration")
    }

    pub fn save(&self) -> Result<()> {
        confy::store("minecli", "config", self).context("Failed to save configuration")
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty() && !self.redmine_url.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.redmine_url, "");
        assert_eq!(config.api_key, "");
        assert!(!config.is_configured());
    }

    #[test]
    fn test_is_configured() {
        let mut config = Config::default();
        assert!(!config.is_configured());

        config.api_key = "test_key".to_string();
        config.redmine_url = "https://example.com/redmine";
        assert!(config.is_configured());

        config.api_key = String::new();
        config.redmine_url = String::new();
        assert!(!config.is_configured());
    }

    #[test]
    fn test_config_with_custom_url() {
        let config = Config {
            redmine_url: "https://example.com/redmine".to_string(),
            api_key: "my_api_key".to_string(),
            theme: ThemeName::default(),
            exclude_subprojects: true,
        };
        assert!(config.is_configured());
        assert_eq!(config.redmine_url, "https://example.com/redmine");
    }
}
