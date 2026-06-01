use ahab::models::Epic;

#[test]
fn test_epic_parsing_full_format() {
    let markdown = r#"# User Authentication

## Description

Implement basic user authentication with email and password.

## Acceptance Criteria

- Users can register with email/password
- Users can log in with valid credentials
- Users can log out
- Invalid credentials return appropriate error

## Technical Notes

Use bcrypt for password hashing.
Implement JWT tokens for session management.
Store tokens in HTTP-only cookies.

## Tags

- authentication
- security

## Labels

- backend
- high-priority
"#;

    let epic = Epic::from_markdown(markdown).unwrap();

    assert_eq!(epic.title, "User Authentication");
    assert!(epic.description.contains("email and password"));
    assert_eq!(epic.acceptance_criteria.as_ref().unwrap().len(), 4);
    assert!(epic.technical_notes.is_some());
    assert_eq!(epic.tags.as_ref().unwrap().len(), 2);
    assert_eq!(epic.labels.as_ref().unwrap().len(), 2);
}

#[test]
fn test_epic_parsing_minimal_format() {
    let markdown = r#"# Minimal Epic

This is just a description with no other sections.
"#;

    let epic = Epic::from_markdown(markdown).unwrap();

    assert_eq!(epic.title, "Minimal Epic");
    assert!(epic.description.contains("just a description"));
    assert!(epic.acceptance_criteria.is_none() || epic.acceptance_criteria.as_ref().unwrap().is_empty());
}

#[test]
fn test_epic_roundtrip() {
    let original = Epic::new(
        "Test Epic".to_string(),
        "Test description".to_string(),
    )
    .with_acceptance_criteria(vec!["Test criterion".to_string()])
    .with_technical_notes("Test notes".to_string())
    .with_tags(vec!["test".to_string()])
    .with_labels(vec!["backend".to_string()]);

    let markdown = original.to_markdown();
    let parsed = Epic::from_markdown(&markdown).unwrap();

    assert_eq!(parsed.title, original.title);
    assert_eq!(parsed.description.trim(), original.description.trim());
    assert_eq!(parsed.acceptance_criteria, original.acceptance_criteria);
    assert_eq!(parsed.tags, original.tags);
    assert_eq!(parsed.labels, original.labels);
}
