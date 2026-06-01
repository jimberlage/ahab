use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::{AhabError, Result};
use crate::models::Epic;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub page_id: Option<String>,
    pub page_name: Option<String>,
    pub page_url: Option<String>,
    pub profile: String,
    pub source: SessionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionSource {
    Page,
    Stdin,
}

pub struct Session {
    pub session_id: String,
    pub session_dir: PathBuf,
    pub metadata: SessionMetadata,
}

impl Session {
    pub fn new(sessions_dir: &PathBuf, profile: String, source: SessionSource) -> Result<Self> {
        let session_id = Uuid::new_v4().to_string();
        Self::with_id(sessions_dir, session_id, profile, source)
    }

    pub fn with_id(
        sessions_dir: &PathBuf,
        session_id: String,
        profile: String,
        source: SessionSource,
    ) -> Result<Self> {
        let session_dir = sessions_dir.join(&session_id);
        fs::create_dir_all(&session_dir)?;

        let metadata = SessionMetadata {
            session_id: session_id.clone(),
            created_at: Utc::now(),
            page_id: None,
            page_name: None,
            page_url: None,
            profile,
            source,
        };

        Ok(Session {
            session_id,
            session_dir,
            metadata,
        })
    }

    pub fn load(sessions_dir: &PathBuf, session_id: &str) -> Result<Self> {
        let session_dir = sessions_dir.join(session_id);
        if !session_dir.exists() {
            return Err(AhabError::SessionNotFound(session_id.to_string()));
        }

        let metadata_path = session_dir.join("metadata.toml");
        let metadata_content = fs::read_to_string(&metadata_path).map_err(|_| {
            AhabError::SessionNotFound(format!("metadata for session {}", session_id))
        })?;
        let metadata: SessionMetadata = toml::from_str(&metadata_content)?;

        Ok(Session {
            session_id: session_id.to_string(),
            session_dir,
            metadata,
        })
    }

    pub fn list_all(sessions_dir: &PathBuf) -> Result<Vec<SessionMetadata>> {
        let mut sessions = Vec::new();

        if !sessions_dir.exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(sessions_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let session_id = entry
                    .file_name()
                    .to_string_lossy()
                    .to_string();
                
                if let Ok(session) = Self::load(sessions_dir, &session_id) {
                    sessions.push(session.metadata);
                }
            }
        }

        // Sort by creation time, newest first
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(sessions)
    }

    pub fn delete(sessions_dir: &PathBuf, session_id: &str) -> Result<()> {
        let session_dir = sessions_dir.join(session_id);
        if !session_dir.exists() {
            return Err(AhabError::SessionNotFound(session_id.to_string()));
        }

        fs::remove_dir_all(&session_dir)?;
        Ok(())
    }

    pub fn set_page_info(&mut self, page_id: String, page_name: String, page_url: Option<String>) {
        self.metadata.page_id = Some(page_id);
        self.metadata.page_name = Some(page_name);
        self.metadata.page_url = page_url;
    }

    pub fn save_metadata(&self) -> Result<()> {
        let metadata_path = self.session_dir.join("metadata.toml");
        let content = toml::to_string_pretty(&self.metadata)?;
        fs::write(metadata_path, content)?;
        Ok(())
    }

    pub fn save_epic(&self, epic: &Epic, filename: &str) -> Result<()> {
        let epic_path = self.session_dir.join(filename);
        let content = epic.to_markdown();
        fs::write(epic_path, content)?;
        Ok(())
    }

    pub fn save_epics(&self, epics: &[Epic]) -> Result<()> {
        for (i, epic) in epics.iter().enumerate() {
            let filename = format!("epic_{:03}.md", i + 1);
            self.save_epic(epic, &filename)?;
        }
        Ok(())
    }

    pub fn load_epics(&self) -> Result<Vec<Epic>> {
        let mut epics = Vec::new();
        let mut entries: Vec<_> = fs::read_dir(&self.session_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "md")
                    .unwrap_or(false)
            })
            .collect();

        // Sort by filename to maintain order
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let content = fs::read_to_string(entry.path())?;
            match Epic::from_markdown(&content) {
                Ok(epic) => epics.push(epic),
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse epic from {}: {}",
                        entry.path().display(),
                        e
                    );
                }
            }
        }

        Ok(epics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_creation() {
        let temp_dir = TempDir::new().unwrap();
        let sessions_dir = temp_dir.path().to_path_buf();

        let session =
            Session::new(&sessions_dir, "default".to_string(), SessionSource::Page).unwrap();

        assert!(session.session_dir.exists());
    }

    #[test]
    fn test_session_save_and_load_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let sessions_dir = temp_dir.path().to_path_buf();

        let mut session =
            Session::new(&sessions_dir, "default".to_string(), SessionSource::Stdin).unwrap();
        session.set_page_info(
            "PAGE-1".to_string(),
            "Test Page".to_string(),
            Some("https://example.com".to_string()),
        );
        session.save_metadata().unwrap();

        let loaded = Session::load(&sessions_dir, &session.session_id).unwrap();
        assert_eq!(loaded.metadata.page_id, Some("PAGE-1".to_string()));
        assert_eq!(loaded.metadata.page_name, Some("Test Page".to_string()));
    }

    #[test]
    fn test_session_save_and_load_epics() {
        let temp_dir = TempDir::new().unwrap();
        let sessions_dir = temp_dir.path().to_path_buf();

        let session =
            Session::new(&sessions_dir, "default".to_string(), SessionSource::Page).unwrap();

        let epics = vec![
            Epic::new("Epic 1".to_string(), "Description 1".to_string()),
            Epic::new("Epic 2".to_string(), "Description 2".to_string()),
        ];

        session.save_epics(&epics).unwrap();
        let loaded_epics = session.load_epics().unwrap();

        assert_eq!(loaded_epics.len(), 2);
        assert_eq!(loaded_epics[0].title, "Epic 1");
        assert_eq!(loaded_epics[1].title, "Epic 2");
    }
}
