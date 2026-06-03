use std::process::Command;

use crate::error::{AhabError, Result};
use crate::session::Session;

pub async fn critique(session_id: String) -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| AhabError::Config(
        "Could not find home directory".to_string()
    ))?;
    let sessions_dir = home_dir.join(".ahab").join("sessions");

    // Load the session to verify it exists
    let session = Session::load(&sessions_dir, &session_id)?;
    
    println!("Launching OpenCode for session: {}", session_id);
    println!("Session directory: {}", session.session_dir.display());

    // Launch opencode with the session directory as the working directory
    let status = Command::new("opencode")
        .current_dir(&session.session_dir)
        .status()
        .map_err(|e| AhabError::Config(
            format!("Failed to launch opencode: {}. Make sure opencode is installed and in your PATH.", e)
        ))?;

    if !status.success() {
        return Err(AhabError::Config(
            format!("OpenCode exited with status: {}", status)
        ));
    }

    Ok(())
}
