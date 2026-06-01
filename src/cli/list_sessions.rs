use crate::config::ConfigManager;
use crate::error::Result;
use crate::session::Session;

pub async fn list_sessions() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let sessions_dir = config_manager.sessions_dir();

    let sessions = Session::list_all(&sessions_dir)?;

    if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }

    println!("Found {} sessions:\n", sessions.len());

    for session in sessions {
        println!("Session ID: {}", session.session_id);
        println!(
            "  Created: {}",
            session.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!("  Profile: {}", session.profile);
        println!("  Source: {:?}", session.source);

        if let Some(page_name) = session.page_name {
            println!("  Page: {}", page_name);
        }
        if let Some(page_id) = session.page_id {
            println!("  Page ID: {}", page_id);
        }
        if let Some(url) = session.page_url {
            println!("  URL: {}", url);
        }

        println!();
    }

    Ok(())
}
