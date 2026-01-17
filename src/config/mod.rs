use crate::error::GhbareError;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub root: String,
    #[serde(default = "default_clone_method")]
    #[allow(dead_code)]
    pub clone_method: String,
    #[serde(default = "default_post_clone_commands")]
    pub post_clone_commands: String,
    pub suffix: Option<String>,
}

fn default_clone_method() -> String {
    "ssh".to_string()
}

fn default_post_clone_commands() -> String {
    r#"echo 'gitdir: .bare' > .git
git config --file .bare/config remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'
git fetch origin
HEAD_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@'); [ -n "$HEAD_BRANCH" ] && git worktree add "$HEAD_BRANCH" "$HEAD_BRANCH""#
        .to_string()
}

pub fn get_config_dir() -> Result<PathBuf, GhbareError> {
    // Use XDG_CONFIG_HOME or default to ~/.config
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg_config).join("ghqb"));
    }

    dirs::home_dir()
        .map(|h| h.join(".config").join("ghqb"))
        .ok_or(GhbareError::ConfigNotFound(
            "Could not determine config directory".to_string(),
        ))
}

pub fn get_config_path() -> Result<PathBuf, GhbareError> {
    Ok(get_config_dir()?.join("config.toml"))
}

pub fn get_config() -> Result<Config, GhbareError> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Err(GhbareError::ConfigNotFound(format!(
            "Config file not found: {}\nRun 'ghqb config' to create it.",
            config_path.display()
        )));
    }

    let content = fs::read_to_string(&config_path)?;
    let config: Config =
        toml::from_str(&content).map_err(|e| GhbareError::ConfigParseError(e.to_string()))?;

    Ok(config)
}

pub fn get_root() -> Result<PathBuf, GhbareError> {
    let config = get_config()?;
    Ok(expand_tilde(&config.root))
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

pub fn default_config_content() -> &'static str {
    r#"# ghqb configuration file

# Repository root directory (required)
root = "~/repos"

# Default clone method: "ssh" or "https"
clone_method = "ssh"

# Commands to run after bare clone (executed in project directory, line by line)
post_clone_commands = '''
echo 'gitdir: .bare' > .git
git config --file .bare/config remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'
git fetch origin
HEAD_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@'); [ -n "$HEAD_BRANCH" ] && git worktree add "$HEAD_BRANCH" "$HEAD_BRANCH"
'''

# Optional: suffix for cloned directory (e.g., ".work" -> repo.work)
# suffix = ".work"
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde_with_path() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_tilde("~/repos"), home.join("repos"));
    }

    #[test]
    fn test_expand_tilde_only() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_tilde("~"), home);
    }

    #[test]
    fn test_expand_absolute_path() {
        assert_eq!(
            expand_tilde("/absolute/path"),
            PathBuf::from("/absolute/path")
        );
    }

    #[test]
    fn test_default_config_content_is_valid_toml() {
        let content = default_config_content();
        let config: Result<Config, _> = toml::from_str(content);
        assert!(config.is_ok());
    }
}
