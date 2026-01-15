use crate::config::{default_config_content, get_config_dir, get_config_path};
use crate::error::GhbareError;
use std::fs;
use std::process::Command;

pub fn execute() -> anyhow::Result<()> {
    let config_dir = get_config_dir()?;
    let config_path = get_config_path()?;

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        println!("Created config directory: {}", config_dir.display());
    }

    // Create default config file if it doesn't exist
    if !config_path.exists() {
        fs::write(&config_path, default_config_content())?;
        println!("Created config file: {}", config_path.display());
    }

    // Get editor from environment
    let editor = std::env::var("EDITOR").map_err(|_| GhbareError::EditorNotFound)?;

    // Open config file with editor
    let status = Command::new(&editor)
        .arg(&config_path)
        .status()
        .map_err(|e| GhbareError::IoError(e))?;

    if !status.success() {
        eprintln!("Editor exited with non-zero status");
    }

    Ok(())
}
