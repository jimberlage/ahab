use crate::api::AhaClient;
use crate::config::ConfigManager;
use crate::error::{AhabError, Result};
use crate::session::Session;

pub async fn accept(session_id: String, profile_name: Option<String>) -> Result<()> {
    let profile = profile_name.unwrap_or_else(|| "default".to_string());
    let config_manager = ConfigManager::new()?;

    // Load session
    let sessions_dir = config_manager.sessions_dir();
    let session = Session::load(&sessions_dir, &session_id)?;

    // Load credentials and config
    let credentials = config_manager.get_profile_credentials(&profile)?;
    let config = config_manager.get_profile_config(&profile)?;

    let aha_domain = config.aha_domain.clone().ok_or_else(|| {
        AhabError::Config("Aha domain is required. Run 'ahab configure' first.".to_string())
    })?;

    let product_id = config.product_id.clone().ok_or_else(|| {
        AhabError::Config("Product ID is required. Run 'ahab configure' first.".to_string())
    })?;

    // Load epics from session
    let epics = session.load_epics()?;

    if epics.is_empty() {
        return Err(AhabError::Session(
            "No epics found in session".to_string(),
        ));
    }

    println!("Creating {} epics in Aha...", epics.len());
    println!();

    // Create Aha client
    let aha_client = AhaClient::new(credentials.aha_token, aha_domain);

    // Track results
    let mut created_urls = Vec::new();
    let mut failures = Vec::new();

    // Create epics
    for (i, epic) in epics.iter().enumerate() {
        print!("Creating epic {}/{}: {}... ", i + 1, epics.len(), epic.title);

        match aha_client.create_epic(&product_id, epic).await {
            Ok(url) => {
                println!("✓");
                created_urls.push((epic.title.clone(), url));
            }
            Err(e) => {
                println!("✗");
                failures.push((epic.title.clone(), e.to_string()));
                tracing::error!("Failed to create epic '{}': {}", epic.title, e);
            }
        }
    }

    println!();

    // Report results
    if !created_urls.is_empty() {
        println!("Successfully created {} epics:", created_urls.len());
        for (title, url) in &created_urls {
            println!("  - {}: {}", title, url);
        }
        println!();
    }

    if !failures.is_empty() {
        println!("Failed to create {} epics:", failures.len());
        for (title, error) in &failures {
            println!("  - {}: {}", title, error);
        }
        println!();
    }

    // Add comment to original page if available
    if let Some(page_id) = &session.metadata.page_id {
        if !created_urls.is_empty() {
            let mut comment = String::from("Created the following epics:\n\n");
            for (title, url) in &created_urls {
                comment.push_str(&format!("- [{}]({})\n", title, url));
            }

            match aha_client.add_comment_to_page(page_id, &comment).await {
                Ok(_) => {
                    println!("Added comment to source page with epic links.");
                }
                Err(e) => {
                    tracing::warn!("Failed to add comment to page: {}", e);
                    println!("Warning: Could not add comment to source page.");
                }
            }
        }
    }

    // Return appropriate exit code
    if failures.is_empty() {
        println!("All epics created successfully!");
        Ok(())
    } else if created_urls.is_empty() {
        Err(AhabError::Api("Failed to create any epics".to_string()))
    } else {
        Err(AhabError::PartialFailure {
            completed: created_urls.len(),
            total: epics.len(),
        })
    }
}
