use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::{AhabError, Result};
use crate::models::Epic;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageManifestEntry {
    pub page_id: String,
    pub page_name: String,
    pub page_url: Option<String>,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicManifestEntry {
    pub filename: String,
    pub epic_id: Option<String>,
    pub epic_url: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub page_id: Option<String>,
    pub page_name: Option<String>,
    pub page_url: Option<String>,
    pub profile: String,
    pub source: SessionSource,
    #[serde(default)]
    pub page_manifest: HashMap<String, PageManifestEntry>,
    #[serde(default)]
    pub epic_manifest: HashMap<String, EpicManifestEntry>,
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

        // Create .opencode directory and breakdown agent configuration
        let opencode_dir = session_dir.join(".opencode");
        fs::create_dir_all(&opencode_dir)?;
        
                let breakdown_agent_config = r#"---
description: Breaks down *.page.md files into epic files (*.epic.md)
mode: subagent
---

You are a technical breakdown agent for the Ahab project.

First, read the metadata.toml file in the current directory to get the session_id and other context.
This will help you understand which session you're working in.

Your role is to analyze *.page.md files in this session directory and break them down into technical epics.

Guidelines:
- Read metadata.toml first to understand the session context
- Read all *.page.md files in the session directory
- Create *.epic.md files that contain detailed technical implementation plans
- Prioritize technical detail over use cases - use cases indicate intent, but epics should be authoritative on implementation
- Multiple features may be included in a single epic if they're closely related
- Each epic should follow this format:
    # Epic Title

    ## Description

    Epic description here...

    ## Acceptance Criteria

    - Criterion 1
    - Criterion 2

    ## Technical Notes

    Technical implementation details...

When you're done, save the epic files as epic_001.epic.md, epic_002.epic.md, etc.
"#;

        // Create agents directory
        let agents_dir = opencode_dir.join("agents");
        fs::create_dir_all(&agents_dir)?;
        
        let agent_config_path = agents_dir.join("breakdown.md");
        fs::write(agent_config_path, breakdown_agent_config)?;
        
        // Create critic agent configuration (primary agent)
        let critic_agent_config = r#"---
description: Reviews, modifies, and manages epics in conversation with the user
mode: primary
---

You are the Critic agent for the Ahab project.

First, read the metadata.toml file in the current directory to get the session_id, profile, and other session context.

Your role is to:
1. Answer questions about created epics in this session
2. Modify epics in response to user concerns and feedback
3. Review epic quality and provide suggestions
4. Push finished epics to Aha when the user requests it
5. Delegate to the breakdown subagent when you need to create new epics from page files

Key behaviors:
- Read metadata.toml first to understand the session context
- Read all *.epic.md files to understand the current state of epics
- Read all *.page.md files to understand the source requirements
- When modifying epics, maintain the standard epic format:
    # Epic Title

    ## Description

    Epic description here...

    ## Acceptance Criteria

    - Criterion 1
    - Criterion 2

    ## Technical Notes

    Technical implementation details...

- When the user asks you to push or save epics to Aha, use the ahab MCP server to call the push command
- When the user asks you to create new epics from pages, delegate to the @breakdown subagent
- Prioritize technical detail over use cases - epics should be authoritative on implementation
- Be conversational and collaborative - this is an iterative refinement process

Available tools:
- Use the ahab MCP server (already configured) to push epics
- Delegate to @breakdown when you need to create new epics from pages
- Read and modify *.epic.md files directly

When pushing epics:
- Use the ahab MCP server's push_session tool with the current session_id from metadata.toml
- The command will only push new epics that haven't been uploaded yet
- After pushing, inform the user of the epic URLs created
"#;

        let critic_agent_path = agents_dir.join("critic.md");
        fs::write(critic_agent_path, critic_agent_config)?;

        // Get the path to the ahab binary (or use "ahab" if not found)
        let ahab_bin = std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "ahab".to_string());

        // Create opencode.jsonc with MCP server configuration
        let opencode_config = format!(r#"{{
  "$schema": "https://opencode.ai/config.json",
  // OpenCode configuration for Ahab session {}
  "mcp": {{
    "ahab": {{
      "type": "local",
      "command": ["{}", "mcp"],
      "description": "Ahab CLI helper for managing Aha tickets"
    }}
  }}
}}
"#, session_id, ahab_bin);
        
        let opencode_config_path = session_dir.join("opencode.jsonc");
        fs::write(opencode_config_path, opencode_config)?;

        // Run specify init to set up specification tracking
        let specify_result = std::process::Command::new("specify")
            .arg("init")
            .arg(&session_id)
            .arg("--integration")
            .arg("opencode")
            .current_dir(&session_dir)
            .output();
        
        match specify_result {
            Ok(output) if output.status.success() => {
                eprintln!("Initialized specify for session {}", session_id);
            }
            Ok(output) => {
                eprintln!("Warning: specify init failed: {}", 
                    String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                eprintln!("Warning: Could not run specify init (is specify installed?): {}", e);
            }
        }

        let metadata = SessionMetadata {
            session_id: session_id.clone(),
            created_at: Utc::now(),
            page_id: None,
            page_name: None,
            page_url: None,
            profile,
            source,
            page_manifest: HashMap::new(),
            epic_manifest: HashMap::new(),
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
                let session_id = entry.file_name().to_string_lossy().to_string();

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

    pub fn load_epics(&self) -> Result<Vec<Epic>> {
        let epics_with_files = self.load_epics_with_filenames()?;
        Ok(epics_with_files.into_iter().map(|(epic, _)| epic).collect())
    }

    pub fn load_epics_with_filenames(&self) -> Result<Vec<(Epic, String)>> {
        let mut epics = Vec::new();
        let mut entries: Vec<_> = fs::read_dir(&self.session_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.ends_with(".epic.md"))
                    .unwrap_or(false)
            })
            .collect();

        // Sort by filename to maintain order
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let filename = entry
                .file_name()
                .to_string_lossy()
                .to_string();
            let content = fs::read_to_string(entry.path())?;
            match Epic::from_markdown(&content) {
                Ok(epic) => epics.push((epic, filename)),
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
        
        // Verify .opencode directory was created
        let opencode_dir = session.session_dir.join(".opencode");
        assert!(opencode_dir.exists());
        
        // Verify breakdown agent config was created
        let breakdown_config = opencode_dir.join("agents").join("breakdown.md");
        assert!(breakdown_config.exists());
        
        // Verify the content of the breakdown config
        let config_content = fs::read_to_string(breakdown_config).unwrap();
        assert!(config_content.contains("description:"));
        assert!(config_content.contains("mode: subagent"));
        assert!(config_content.contains("*.page.md"));
        assert!(config_content.contains("*.epic.md"));
        assert!(config_content.contains("metadata.toml"));
        
        // Verify critic agent config was created
        let critic_config = opencode_dir.join("agents").join("critic.md");
        assert!(critic_config.exists());
        
        // Verify the content of the critic config
        let critic_content = fs::read_to_string(critic_config).unwrap();
        assert!(critic_content.contains("description:"));
        assert!(critic_content.contains("mode: primary"));
        assert!(critic_content.contains("*.epic.md"));
        assert!(critic_content.contains("metadata.toml"));
        assert!(critic_content.contains("@breakdown"));
        
        // Verify opencode.jsonc was created
        let opencode_jsonc = session.session_dir.join("opencode.jsonc");
        assert!(opencode_jsonc.exists());
        
        // Verify the content of opencode.jsonc
        let jsonc_content = fs::read_to_string(opencode_jsonc).unwrap();
        assert!(jsonc_content.contains("\"mcp\""));
        assert!(jsonc_content.contains("\"ahab\""));
        assert!(jsonc_content.contains("\"type\": \"local\""));
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

        // Save epics individually
        for (i, epic) in epics.iter().enumerate() {
            let filename = format!("epic_{:03}.epic.md", i + 1);
            session.save_epic(epic, &filename).unwrap();
        }
        
        let loaded_epics = session.load_epics().unwrap();

        assert_eq!(loaded_epics.len(), 2);
        assert_eq!(loaded_epics[0].title, "Epic 1");
        assert_eq!(loaded_epics[1].title, "Epic 2");
    }
}
