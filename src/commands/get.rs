use crate::config;
use crate::error::GhbareError;
use crate::git;
use crate::url::{parse_repo_url, RepoInfo};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn execute(repo: &str, ssh: bool, https: bool, suffix: Option<String>) -> anyhow::Result<()> {
    let repo_info = parse_repo_url(repo)?;
    println!(
        "Repository: {}/{}/{}",
        repo_info.host, repo_info.owner, repo_info.repo
    );

    let cfg = config::get_config()?;
    let clone_url = determine_clone_url(&repo_info, ssh, https)?;
    println!("Clone URL: {}", clone_url);

    let root = config::get_root()?;

    // Determine suffix: CLI option > config > none
    let effective_suffix = suffix.or(cfg.suffix.clone());

    let local_path = match &effective_suffix {
        Some(s) => format!("{}{}", repo_info.to_local_path(), s),
        None => repo_info.to_local_path(),
    };

    let project_dir = root.join(&local_path);
    let bare_dir = project_dir.join(".bare");

    if project_dir.exists() {
        return Err(GhbareError::RepositoryAlreadyExists(project_dir.display().to_string()).into());
    }

    fs::create_dir_all(&project_dir)?;
    println!("Created: {}", project_dir.display());

    println!("Cloning into {}...", bare_dir.display());
    git::bare_clone(&clone_url, &bare_dir)?;

    // Run post_clone_commands in project directory
    run_post_clone_commands(&cfg.post_clone_commands, &project_dir)?;

    // Create empty .envrc
    let envrc_path = project_dir.join(".envrc");
    fs::write(&envrc_path, "")?;
    println!("Created .envrc");

    println!("\nDone! Repository cloned to: {}", project_dir.display());

    Ok(())
}

fn run_post_clone_commands(commands: &str, working_dir: &Path) -> Result<(), GhbareError> {
    if commands.trim().is_empty() {
        return Ok(());
    }
    println!("Running post-clone commands...");
    let status = Command::new("sh")
        .arg("-c")
        .arg(commands)
        .current_dir(working_dir)
        .status()
        .map_err(|e| GhbareError::PostCloneCommandError(format!("Failed to execute: {}", e)))?;
    if !status.success() {
        return Err(GhbareError::PostCloneCommandError(
            "Post-clone commands failed".to_string(),
        ));
    }
    Ok(())
}

fn determine_clone_url(repo_info: &RepoInfo, ssh: bool, https: bool) -> Result<String, GhbareError> {
    match (ssh, https) {
        (true, true) => Err(GhbareError::UrlParseError(
            "Cannot specify both --ssh and --https".to_string(),
        )),
        (true, false) => Ok(repo_info.to_ssh_url()),
        (false, true) => Ok(repo_info.to_https_url()),
        (false, false) => Ok(repo_info.to_ssh_url()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_clone_url_ssh() {
        let info = RepoInfo {
            host: "github.com".to_string(),
            owner: "user".to_string(),
            repo: "repo".to_string(),
        };
        let url = determine_clone_url(&info, true, false).unwrap();
        assert_eq!(url, "git@github.com:user/repo.git");
    }

    #[test]
    fn test_determine_clone_url_https() {
        let info = RepoInfo {
            host: "github.com".to_string(),
            owner: "user".to_string(),
            repo: "repo".to_string(),
        };
        let url = determine_clone_url(&info, false, true).unwrap();
        assert_eq!(url, "https://github.com/user/repo.git");
    }

    #[test]
    fn test_determine_clone_url_default() {
        let info = RepoInfo {
            host: "github.com".to_string(),
            owner: "user".to_string(),
            repo: "repo".to_string(),
        };
        let url = determine_clone_url(&info, false, false).unwrap();
        assert_eq!(url, "git@github.com:user/repo.git");
    }

    #[test]
    fn test_determine_clone_url_both_error() {
        let info = RepoInfo {
            host: "github.com".to_string(),
            owner: "user".to_string(),
            repo: "repo".to_string(),
        };
        let result = determine_clone_url(&info, true, true);
        assert!(result.is_err());
    }
}
