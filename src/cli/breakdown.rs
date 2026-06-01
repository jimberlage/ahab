use std::io::{self, Read};

use crate::api::{AhaClient, OpenRouterClient};
use crate::config::ConfigManager;
use crate::error::{AhabError, Result};
use crate::models::Epic;
use crate::session::{Session, SessionSource};

pub async fn breakdown(
    page_id: Option<String>,
    profile_name: Option<String>,
    session_id: Option<String>,
    use_stdin: bool,
) -> Result<()> {
    let profile = profile_name.unwrap_or_else(|| "default".to_string());
    let config_manager = ConfigManager::new()?;

    // Load credentials and config
    let credentials = config_manager.get_profile_credentials(&profile)?;
    let config = config_manager.get_profile_config(&profile)?;

    // Validate configuration
    if credentials.openrouter_api_key.is_none() {
        return Err(AhabError::Config(
            "OpenRouter API key is required for breakdown. Run 'ahab configure' first.".to_string(),
        ));
    }

    let aha_domain = config.aha_domain.clone().ok_or_else(|| {
        AhabError::Config("Aha domain is required. Run 'ahab configure' first.".to_string())
    })?;

    let default_model = config
        .default_model
        .clone()
        .unwrap_or_else(|| "anthropic/claude-sonnet-4".to_string());

    // Determine source and get content
    let (content, source, page_info) = if use_stdin {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        (buffer, SessionSource::Stdin, None)
    } else if let Some(pid) = page_id {
        // Fetch from Aha
        let aha_client = AhaClient::new(credentials.aha_token.clone(), aha_domain.clone());
        let page = aha_client.get_page(&pid).await?;
        let content = page.to_markdown();
        (
            content,
            SessionSource::Page,
            Some((page.id, page.name, page.url)),
        )
    } else {
        return Err(AhabError::InvalidInput(
            "Either --page-id or --stdin must be provided".to_string(),
        ));
    };

    // Create or load session
    let sessions_dir = config_manager.sessions_dir();
    let mut session = if let Some(sid) = session_id {
        Session::with_id(&sessions_dir, sid, profile.clone(), source)?
    } else {
        Session::new(&sessions_dir, profile.clone(), source)?
    };

    // Set page info if available
    if let Some((id, name, url)) = page_info {
        session.set_page_info(id, name, url);
    }

    println!("Breaking down content using {}...", default_model);
    println!();

    // Call OpenRouter to generate epics
    let openrouter_client = OpenRouterClient::new(
        credentials.openrouter_api_key.unwrap(),
        default_model,
    );

    let epics_markdown = openrouter_client.breakdown_to_epics(&content).await?;

    // Parse the response into individual epics
    let epic_sections: Vec<&str> = epics_markdown.split("\n---\n").collect();
    let mut epics = Vec::new();

    for section in epic_sections {
        let trimmed = section.trim();
        if !trimmed.is_empty() {
            match Epic::from_markdown(trimmed) {
                Ok(epic) => epics.push(epic),
                Err(e) => {
                    tracing::warn!("Failed to parse epic: {}", e);
                }
            }
        }
    }

    if epics.is_empty() {
        return Err(AhabError::Api(
            "No valid epics were generated".to_string(),
        ));
    }

    // Save epics and metadata
    session.save_epics(&epics)?;
    session.save_metadata()?;

    println!("Generated {} epics", epics.len());
    println!();
    println!("Session ID: {}", session.session_id);
    println!("Session directory: {}", session.session_dir.display());
    println!();
    println!("Epic titles:");
    for (i, epic) in epics.iter().enumerate() {
        println!("  {}. {}", i + 1, epic.title);
    }
    println!();
    println!("Review the generated epics in the session directory.");
    println!(
        "Run 'ahab accept --session {}' to create these epics in Aha.",
        session.session_id
    );

    Ok(())
}
