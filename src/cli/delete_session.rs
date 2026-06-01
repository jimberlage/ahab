use crate::config::ConfigManager;
use crate::error::Result;
use crate::session::Session;

pub async fn delete_session(session_id: String) -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let sessions_dir = config_manager.sessions_dir();

    Session::delete(&sessions_dir, &session_id)?;

    println!("Session {} deleted successfully.", session_id);

    Ok(())
}
