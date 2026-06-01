use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epic {
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Option<Vec<String>>,
    pub technical_notes: Option<String>,
    pub tags: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
}

impl Epic {
    pub fn new(title: String, description: String) -> Self {
        Epic {
            title,
            description,
            acceptance_criteria: None,
            technical_notes: None,
            tags: None,
            labels: None,
        }
    }

    pub fn with_acceptance_criteria(mut self, criteria: Vec<String>) -> Self {
        self.acceptance_criteria = Some(criteria);
        self
    }

    pub fn with_technical_notes(mut self, notes: String) -> Self {
        self.technical_notes = Some(notes);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Parse an epic from markdown content
    pub fn from_markdown(content: &str) -> crate::error::Result<Self> {
        let lines = content.lines().peekable();
        let mut title = String::new();
        let mut description = String::new();
        let mut acceptance_criteria = Vec::new();
        let mut technical_notes = String::new();
        let mut tags = Vec::new();
        let mut labels = Vec::new();

        let mut current_section = Section::Title;

        for line in lines {
            let trimmed = line.trim();

            // Section headers
            if trimmed.starts_with("# ") {
                title = trimmed[2..].to_string();
                current_section = Section::Description;
                continue;
            } else if trimmed.eq_ignore_ascii_case("## description") {
                current_section = Section::Description;
                continue;
            } else if trimmed.eq_ignore_ascii_case("## acceptance criteria") {
                current_section = Section::AcceptanceCriteria;
                continue;
            } else if trimmed.eq_ignore_ascii_case("## technical notes") {
                current_section = Section::TechnicalNotes;
                continue;
            } else if trimmed.eq_ignore_ascii_case("## tags") {
                current_section = Section::Tags;
                continue;
            } else if trimmed.eq_ignore_ascii_case("## labels") {
                current_section = Section::Labels;
                continue;
            }

            // Content
            match current_section {
                Section::Title => {
                    if !trimmed.is_empty() {
                        title = trimmed.to_string();
                    }
                }
                Section::Description => {
                    if !trimmed.is_empty() {
                        description.push_str(line);
                        description.push('\n');
                    }
                }
                Section::AcceptanceCriteria => {
                    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                        acceptance_criteria.push(trimmed[2..].to_string());
                    }
                }
                Section::TechnicalNotes => {
                    if !trimmed.is_empty() {
                        technical_notes.push_str(line);
                        technical_notes.push('\n');
                    }
                }
                Section::Tags => {
                    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                        tags.push(trimmed[2..].to_string());
                    }
                }
                Section::Labels => {
                    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                        labels.push(trimmed[2..].to_string());
                    }
                }
            }
        }

        if title.is_empty() {
            return Err(crate::error::AhabError::InvalidInput(
                "Epic title is required".to_string(),
            ));
        }

        let mut epic = Epic::new(title, description.trim().to_string());

        if !acceptance_criteria.is_empty() {
            epic = epic.with_acceptance_criteria(acceptance_criteria);
        }
        if !technical_notes.trim().is_empty() {
            epic = epic.with_technical_notes(technical_notes.trim().to_string());
        }
        if !tags.is_empty() {
            epic = epic.with_tags(tags);
        }
        if !labels.is_empty() {
            epic = epic.with_labels(labels);
        }

        Ok(epic)
    }

    /// Convert epic to markdown format
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {}\n\n", self.title);

        md.push_str("## Description\n\n");
        md.push_str(&self.description);
        md.push_str("\n\n");

        if let Some(criteria) = &self.acceptance_criteria {
            if !criteria.is_empty() {
                md.push_str("## Acceptance Criteria\n\n");
                for criterion in criteria {
                    md.push_str(&format!("- {}\n", criterion));
                }
                md.push('\n');
            }
        }

        if let Some(notes) = &self.technical_notes {
            if !notes.trim().is_empty() {
                md.push_str("## Technical Notes\n\n");
                md.push_str(notes);
                md.push_str("\n\n");
            }
        }

        if let Some(tags) = &self.tags {
            if !tags.is_empty() {
                md.push_str("## Tags\n\n");
                for tag in tags {
                    md.push_str(&format!("- {}\n", tag));
                }
                md.push('\n');
            }
        }

        if let Some(labels) = &self.labels {
            if !labels.is_empty() {
                md.push_str("## Labels\n\n");
                for label in labels {
                    md.push_str(&format!("- {}\n", label));
                }
                md.push('\n');
            }
        }

        md
    }
}

#[derive(Debug, Clone, Copy)]
enum Section {
    Title,
    Description,
    AcceptanceCriteria,
    TechnicalNotes,
    Tags,
    Labels,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epic_from_markdown() {
        let md = r#"# User Authentication Epic

## Description

Implement user authentication with email and password.

## Acceptance Criteria

- Users can register with email/password
- Users can log in
- Users can log out

## Technical Notes

Use JWT tokens for session management.

## Tags

- authentication
- security

## Labels

- backend
"#;

        let epic = Epic::from_markdown(md).unwrap();
        assert_eq!(epic.title, "User Authentication Epic");
        assert!(epic.description.contains("email and password"));
        assert_eq!(epic.acceptance_criteria.as_ref().unwrap().len(), 3);
        assert!(epic.technical_notes.as_ref().unwrap().contains("JWT"));
    }

    #[test]
    fn test_epic_to_markdown() {
        let epic = Epic::new(
            "Test Epic".to_string(),
            "This is a test".to_string(),
        )
        .with_acceptance_criteria(vec!["Criterion 1".to_string()])
        .with_tags(vec!["test".to_string()]);

        let md = epic.to_markdown();
        assert!(md.contains("# Test Epic"));
        assert!(md.contains("## Description"));
        assert!(md.contains("## Acceptance Criteria"));
    }
}
