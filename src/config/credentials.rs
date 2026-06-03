use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    #[serde(flatten)]
    pub profiles: HashMap<String, ProfileCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCredentials {
    pub aha_token: String,
}

impl Credentials {
    pub fn load(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Credentials {
                profiles: HashMap::new(),
            });
        }

        let content = fs::read_to_string(path)?;
        let credentials: Credentials = toml::from_str(&content)?;
        Ok(credentials)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_profile(&self, profile_name: &str) -> Option<&ProfileCredentials> {
        self.profiles.get(profile_name)
    }

    pub fn set_profile(&mut self, profile_name: String, credentials: ProfileCredentials) {
        self.profiles.insert(profile_name, credentials);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_credentials() {
        let temp_dir = TempDir::new().unwrap();
        let creds_path = temp_dir.path().join("credentials");

        let mut credentials = Credentials {
            profiles: HashMap::new(),
        };
        credentials.set_profile(
            "default".to_string(),
            ProfileCredentials {
                aha_token: "test_token".to_string(),
            },
        );

        credentials.save(&creds_path).unwrap();
        let loaded = Credentials::load(&creds_path).unwrap();

        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(
            loaded.get_profile("default").unwrap().aha_token,
            "test_token"
        );
    }
}
