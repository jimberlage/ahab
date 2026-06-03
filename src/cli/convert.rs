use std::collections::{HashMap, HashSet};

use crate::api::{AhaClient, AhaPage};
use crate::config::ConfigManager;
use crate::error::{AhabError, Result};
use crate::session::{Session, SessionSource};

pub async fn convert(
    pages: Vec<String>,
    profile: Option<String>,
    session_id: Option<String>,
) -> Result<()> {
    if pages.is_empty() {
        return Err(AhabError::InvalidInput(
            "At least one page URL or slug is required".to_string(),
        ));
    }

    // Load configuration
    let config_manager = ConfigManager::new()?;
    let profile_name = profile.unwrap_or_else(|| "default".to_string());

    // Load credentials and config
    let credentials = config_manager.get_profile_credentials(&profile_name)?;
    let profile_config = config_manager.get_profile_config(&profile_name)?;

    let aha_domain = profile_config
        .aha_domain
        .ok_or_else(|| AhabError::InvalidInput("Aha domain not configured".to_string()))?;

    let aha_client = AhaClient::new(credentials.aha_token, aha_domain);

    // Get sessions directory
    let sessions_dir = config_manager.sessions_dir();

    // Create or load session
    let (mut session, session_created) = if let Some(sid) = session_id {
        // Try to load existing session, or create it if it doesn't exist
        match Session::load(&sessions_dir, &sid) {
            Ok(session) => (session, false),
            Err(AhabError::SessionNotFound(_)) => {
                // Create new session with the specified ID
                let session = Session::with_id(
                    &sessions_dir,
                    sid.clone(),
                    profile_name.clone(),
                    SessionSource::Page,
                )?;
                (session, true)
            }
            Err(e) => return Err(e),
        }
    } else {
        (
            Session::new(&sessions_dir, profile_name.clone(), SessionSource::Page)?,
            true,
        )
    };

    if session_created {
        println!("Created new session: {}", session.session_id);
    } else {
        println!("Using existing session: {}", session.session_id);
    }

    // Load existing manifest from metadata
    let mut manifest = session.metadata.page_manifest.clone();

    // Track processed pages to avoid duplicates
    let mut processed_pages = HashSet::new();

    // Cache for all pages by product_id
    let mut product_pages_cache: HashMap<String, Vec<AhaPage>> = HashMap::new();

    // Process each page (and recursively process children)
    for page_input in pages {
        process_page_recursively(
            &page_input,
            &aha_client,
            &session,
            &mut manifest,
            &mut processed_pages,
            &mut product_pages_cache,
        )
        .await?;
    }

    // Update session page info (for backward compatibility with first page)
    if session.metadata.page_id.is_none() && !manifest.is_empty() {
        if let Some((_, first_entry)) = manifest.iter().next() {
            session.set_page_info(
                first_entry.page_id.clone(),
                first_entry.page_name.clone(),
                first_entry.page_url.clone(),
            );
        }
    }

    // Save updated manifest
    session.metadata.page_manifest = manifest;
    session.save_metadata()?;

    println!("\nSession saved: {}", session.session_id);
    println!("Processed {} page(s) total", processed_pages.len());
    println!("\nNext steps:");
    println!("  - Create epics as *.epic.md files in the session directory");
    println!("  - Run: ahab push --session {}", session.session_id);

    Ok(())
}

/// Recursively process a page and all its children
async fn process_page_recursively(
    page_input: &str,
    aha_client: &AhaClient,
    session: &Session,
    manifest: &mut std::collections::HashMap<String, crate::session::PageManifestEntry>,
    processed_pages: &mut HashSet<String>,
    product_pages_cache: &mut HashMap<String, Vec<AhaPage>>,
) -> Result<()> {
    // Parse page slug from URL or use directly
    let page_slug = parse_page_slug(page_input)?;

    // Skip if already processed
    if processed_pages.contains(&page_slug) {
        return Ok(());
    }

    println!("Processing page: {}", page_slug);

    // Fetch page from Aha
    let page = aha_client.get_page(&page_slug).await?;

    // Mark as processed
    processed_pages.insert(page_slug.clone());

    // Convert to markdown
    let markdown = page.to_markdown();

    // Save as *.page.md using reference_num (e.g., VAFM-N-91.page.md)
    let filename = format!("{}.page.md", sanitize_filename(&page.reference_num));
    let filepath = session.session_dir.join(&filename);
    std::fs::write(&filepath, &markdown)?;

    println!("  Saved to: {}", filename);

    // Update manifest
    manifest.insert(
        filename.clone(),
        crate::session::PageManifestEntry {
            page_id: page.id.clone(),
            page_name: page.name.clone(),
            page_url: page.url.clone(),
            filename,
        },
    );

    // Fetch child pages (pages with parent_id = this page's id)
    if let Some(product_id) = &page.product_id {
        // Fetch all pages for this product if not cached
        if !product_pages_cache.contains_key(product_id) {
            let all_pages = aha_client.fetch_all_pages(product_id).await?;
            product_pages_cache.insert(product_id.clone(), all_pages);
        }

        // Get children from cached list
        let all_pages = product_pages_cache.get(product_id).unwrap();
        let child_refs = aha_client.get_children_from_list(all_pages, &page.id);

        if !child_refs.is_empty() {
            println!("  Found {} child page(s)", child_refs.len());
            for child_ref in &child_refs {
                // Recursively process each child
                Box::pin(process_page_recursively(
                    child_ref,
                    aha_client,
                    session,
                    manifest,
                    processed_pages,
                    product_pages_cache,
                ))
                .await?;
            }
        }
    }

    Ok(())
}

/// Parse page slug from URL or raw slug
fn parse_page_slug(input: &str) -> Result<String> {
    // If it's a full URL, extract the slug
    if input.starts_with("http://") || input.starts_with("https://") {
        // Expected format: https://apexlabs.aha.io/pages/VAFM-N-91
        let parts: Vec<&str> = input.split('/').collect();
        if let Some(slug) = parts.last() {
            if !slug.is_empty() {
                return Ok(slug.to_string());
            }
        }
        return Err(AhabError::InvalidInput(format!(
            "Invalid page URL format: {}",
            input
        )));
    }

    // Otherwise, treat it as a slug directly
    Ok(input.to_string())
}

/// Sanitize filename for filesystem
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_page_slug_from_url() {
        let url = "https://apexlabs.aha.io/pages/VAFM-N-91";
        let slug = parse_page_slug(url).unwrap();
        assert_eq!(slug, "VAFM-N-91");
    }

    #[test]
    fn test_parse_page_slug_from_slug() {
        let slug = "VAFM-N-91";
        let result = parse_page_slug(slug).unwrap();
        assert_eq!(result, "VAFM-N-91");
    }

    #[test]
    fn test_sanitize_filename() {
        let name = "test/file:name";
        let sanitized = sanitize_filename(name);
        assert_eq!(sanitized, "test_file_name");
    }
}
