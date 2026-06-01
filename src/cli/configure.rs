use dialoguer::{Input, Password};

use crate::config::credentials::ProfileCredentials;
use crate::config::profile::ProfileConfig;
use crate::config::ConfigManager;
use crate::error::Result;

pub async fn configure(profile_name: Option<String>) -> Result<()> {
    let profile = profile_name.unwrap_or_else(|| "default".to_string());

    println!("Configuring profile: {}", profile);
    println!();

    let config_manager = ConfigManager::new()?;
    config_manager.ensure_config_dir()?;

    // Load existing config
    let mut credentials = config_manager.load_credentials()?;
    let mut config = config_manager.load_config()?;

    // Get existing values or defaults
    let existing_creds = credentials.get_profile(&profile).cloned();
    let existing_config = config.get_profile(&profile).cloned().unwrap_or_default();

    // Prompt for credentials
    let aha_token: String = Input::new()
        .with_prompt("Aha API Token")
        .default(existing_creds.as_ref().map(|c| c.aha_token.clone()).unwrap_or_default())
        .interact_text()?;

    let openrouter_api_key: String = Password::new()
        .with_prompt("OpenRouter API Key (optional, press Enter to skip)")
        .allow_empty_password(true)
        .interact()?;

    // Prompt for config
    let aha_domain: String = Input::new()
        .with_prompt("Aha Domain (e.g., mycompany.aha.io)")
        .default(existing_config.aha_domain.unwrap_or_default())
        .interact_text()?;

    let workspace_id: String = Input::new()
        .with_prompt("Workspace ID (optional, press Enter to skip)")
        .allow_empty(true)
        .default(existing_config.workspace_id.unwrap_or_default())
        .interact_text()?;

    let product_id: String = Input::new()
        .with_prompt("Product ID")
        .default(existing_config.product_id.unwrap_or_default())
        .interact_text()?;

    let team_id: String = Input::new()
        .with_prompt("Team ID (optional, press Enter to skip)")
        .allow_empty(true)
        .default(existing_config.team_id.unwrap_or_default())
        .interact_text()?;

    let default_model: String = Input::new()
        .with_prompt("Default OpenRouter Model")
        .default(
            existing_config
                .default_model
                .unwrap_or_else(|| "anthropic/claude-sonnet-4".to_string()),
        )
        .interact_text()?;

    // Save credentials
    let profile_credentials = ProfileCredentials {
        aha_token,
        openrouter_api_key: if openrouter_api_key.is_empty() {
            None
        } else {
            Some(openrouter_api_key)
        },
    };
    credentials.set_profile(profile.clone(), profile_credentials);
    config_manager.save_credentials(&credentials)?;

    // Save config
    let profile_config = ProfileConfig {
        workspace_id: if workspace_id.is_empty() {
            None
        } else {
            Some(workspace_id)
        },
        product_id: if product_id.is_empty() {
            None
        } else {
            Some(product_id)
        },
        team_id: if team_id.is_empty() { None } else { Some(team_id) },
        default_model: if default_model.is_empty() {
            None
        } else {
            Some(default_model)
        },
        aha_domain: if aha_domain.is_empty() {
            None
        } else {
            Some(aha_domain)
        },
    };
    config.set_profile(profile.clone(), profile_config);
    config_manager.save_config(&config)?;

    println!();
    println!("Configuration saved successfully for profile: {}", profile);

    Ok(())
}
