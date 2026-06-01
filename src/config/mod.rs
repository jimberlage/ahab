pub mod credentials;
pub mod profile;

use std::path::PathBuf;

use crate::error::{AhabError, Result};
use credentials::{Credentials, ProfileCredentials};
use profile::{Config, ProfileConfig};

pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| {
            AhabError::Config("Could not determine home directory".to_string())
        })?;
        let config_dir = home.join(".ahab");
        Ok(ConfigManager { config_dir })
    }

    pub fn with_config_dir(config_dir: PathBuf) -> Self {
        ConfigManager { config_dir }
    }

    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn credentials_path(&self) -> PathBuf {
        self.config_dir.join("credentials")
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("config")
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.config_dir.join("sessions")
    }

    pub fn load_credentials(&self) -> Result<Credentials> {
        Credentials::load(&self.credentials_path())
    }

    pub fn save_credentials(&self, credentials: &Credentials) -> Result<()> {
        credentials.save(&self.credentials_path())
    }

    pub fn load_config(&self) -> Result<Config> {
        Config::load(&self.config_path())
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        config.save(&self.config_path())
    }

    pub fn get_profile_credentials(&self, profile_name: &str) -> Result<ProfileCredentials> {
        let credentials = self.load_credentials()?;
        credentials
            .get_profile(profile_name)
            .cloned()
            .ok_or_else(|| AhabError::ProfileNotFound(profile_name.to_string()))
    }

    pub fn get_profile_config(&self, profile_name: &str) -> Result<ProfileConfig> {
        let config = self.load_config()?;
        Ok(config
            .get_profile(profile_name)
            .cloned()
            .unwrap_or_default())
    }

    pub fn ensure_config_dir(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.sessions_dir())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::with_config_dir(temp_dir.path().to_path_buf());

        manager.ensure_config_dir().unwrap();

        assert!(manager.config_dir().exists());
        assert!(manager.sessions_dir().exists());
    }
}
