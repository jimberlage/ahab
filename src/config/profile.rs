use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub profiles: HashMap<String, ProfileConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileConfig {
    pub workspace_id: Option<String>,
    pub product_id: Option<String>,
    pub team_id: Option<String>,
    pub default_model: Option<String>,
    pub aha_domain: Option<String>,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Config {
                profiles: HashMap::new(),
            });
        }

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_profile(&self, profile_name: &str) -> Option<&ProfileConfig> {
        self.profiles.get(profile_name)
    }

    pub fn set_profile(&mut self, profile_name: String, config: ProfileConfig) {
        self.profiles.insert(profile_name, config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config");

        let mut config = Config {
            profiles: HashMap::new(),
        };
        config.set_profile(
            "default".to_string(),
            ProfileConfig {
                workspace_id: Some("WORKSPACE-1".to_string()),
                product_id: Some("PRODUCT-1".to_string()),
                team_id: None,
                default_model: Some("anthropic/claude-sonnet-4".to_string()),
                aha_domain: Some("mycompany.aha.io".to_string()),
            },
        );

        config.save(&config_path).unwrap();
        let loaded = Config::load(&config_path).unwrap();

        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(
            loaded.get_profile("default").unwrap().workspace_id,
            Some("WORKSPACE-1".to_string())
        );
    }
}
