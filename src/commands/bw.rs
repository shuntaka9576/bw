use crate::error::GhbareError;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Deserialize)]
pub struct BwConfig {
    #[serde(default = "default_base_branch")]
    pub base_branch: String,

    #[serde(default)]
    pub post_add_commands: String,
}

fn default_base_branch() -> String {
    "main".to_string()
}

impl Default for BwConfig {
    fn default() -> Self {
        Self {
            base_branch: default_base_branch(),
            post_add_commands: String::new(),
        }
    }
}

pub fn execute_add(branch: Option<&str>, base_override: Option<String>) -> anyhow::Result<()> {
    let repo_root = find_repo_root()?;
    eprintln!("Repository root: {}", repo_root.display());

    // Clean up stale worktree registrations if needed
    prune_worktrees_if_needed(&repo_root);

    let config = load_bw_config(&repo_root)?;

    let base_branch = base_override.unwrap_or(config.base_branch);

    // ブランチ名の決定: 指定があればそれを使用、なければ自動生成
    let branch = match branch {
        Some(b) => b.to_string(),
        None => {
            let generated = generate_wip_branch_name();
            eprintln!("Auto-generated branch name: {}", generated);
            generated
        }
    };

    let dirname = branch_to_dirname(&branch);
    let worktree_path = repo_root.join(&dirname);

    if worktree_path.exists() {
        return Err(GhbareError::WorktreeAlreadyExists(worktree_path.display().to_string()).into());
    }

    eprintln!(
        "Creating worktree: {} (branch: {}, base: {})",
        dirname, branch, base_branch
    );
    add_worktree(&repo_root, &worktree_path, &branch, &base_branch)?;

    if !config.post_add_commands.is_empty() {
        run_post_add_commands(&config.post_add_commands, &worktree_path)?;
    }

    eprintln!("\nDone! Worktree created at: {}", worktree_path.display());

    Ok(())
}

pub fn execute_list() -> anyhow::Result<()> {
    let repo_root = find_repo_root()?;

    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(&repo_root)
        .output()
        .map_err(|e| GhbareError::WorktreeError(e.to_string()))?;

    if !output.status.success() {
        return Err(GhbareError::WorktreeError("git worktree list failed".to_string()).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let worktrees: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with("worktree "))
        .map(|line| line.strip_prefix("worktree ").unwrap_or(line))
        .collect();

    if worktrees.is_empty() {
        eprintln!("No worktrees found");
        return Ok(());
    }

    let mut fzf = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| GhbareError::WorktreeError(format!("Failed to start fzf: {}", e)))?;

    if let Some(mut stdin) = fzf.stdin.take() {
        for wt in &worktrees {
            writeln!(stdin, "{}", wt)?;
        }
    }

    let output = fzf
        .wait_with_output()
        .map_err(|e| GhbareError::WorktreeError(format!("fzf error: {}", e)))?;

    if output.status.success() {
        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !selected.is_empty() {
            println!("{}", selected);
        }
    }

    Ok(())
}

pub fn execute_rm(name: &str, force: bool) -> anyhow::Result<()> {
    let repo_root = find_repo_root()?;
    let dirname = branch_to_dirname(name);
    let worktree_path = repo_root.join(&dirname);

    if !worktree_path.exists() {
        return Err(
            GhbareError::WorktreeError(format!("Worktree not found: {}", name)).into(),
        );
    }

    eprintln!("Removing worktree: {}", worktree_path.display());

    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(worktree_path.to_str().unwrap());

    let status = Command::new("git")
        .args(&args)
        .current_dir(&repo_root)
        .status()
        .map_err(|e| GhbareError::WorktreeError(e.to_string()))?;

    if !status.success() {
        return Err(GhbareError::WorktreeError(format!(
            "git worktree remove failed for '{}'",
            name
        ))
        .into());
    }

    eprintln!("Done! Worktree removed: {}", name);

    Ok(())
}

fn find_repo_root() -> Result<PathBuf, GhbareError> {
    let current = std::env::current_dir()?;
    let mut dir = current.as_path();

    loop {
        let bare_path = dir.join(".bare");
        if bare_path.exists() && bare_path.is_dir() {
            return Ok(dir.to_path_buf());
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => return Err(GhbareError::RepoRootNotFound),
        }
    }
}

fn load_bw_config(repo_root: &Path) -> Result<BwConfig, GhbareError> {
    let config_path = repo_root.join("bw.toml");

    if !config_path.exists() {
        return Ok(BwConfig::default());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: BwConfig = toml::from_str(&content)
        .map_err(|e| GhbareError::ConfigParseError(e.to_string()))?;

    Ok(config)
}

fn branch_to_dirname(branch: &str) -> String {
    branch.replace('/', "-")
}

fn generate_wip_branch_name() -> String {
    let output = Command::new("date")
        .arg("+%m%d-%H%M%S")
        .output()
        .expect("Failed to execute date command");

    let timestamp = String::from_utf8_lossy(&output.stdout).trim().to_string();
    format!("wip/{}", timestamp)
}

fn prune_worktrees_if_needed(repo_root: &Path) {
    // Check if pruning is needed (output may go to stdout or stderr)
    let output = Command::new("git")
        .args(["worktree", "prune", "--dry-run"])
        .current_dir(repo_root)
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stdout.trim().is_empty() || !stderr.trim().is_empty() {
            eprintln!("Pruning stale worktree entries...");
            let _ = Command::new("git")
                .args(["worktree", "prune"])
                .current_dir(repo_root)
                .status();
        }
    }
}

fn branch_exists(repo_root: &Path, branch: &str) -> bool {
    Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{}", branch),
        ])
        .current_dir(repo_root)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn add_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    branch_name: &str,
    base_branch: &str,
) -> Result<(), GhbareError> {
    let status = if branch_exists(repo_root, branch_name) {
        // 既存ブランチ: git worktree add <path> <branch>
        Command::new("git")
            .args([
                "worktree",
                "add",
                worktree_path.to_str().unwrap(),
                branch_name,
            ])
            .current_dir(repo_root)
            .status()
    } else {
        // 新規ブランチ: git worktree add -b <branch> <path> <base>
        Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                branch_name,
                worktree_path.to_str().unwrap(),
                base_branch,
            ])
            .current_dir(repo_root)
            .status()
    }
    .map_err(|e| GhbareError::WorktreeError(e.to_string()))?;

    if !status.success() {
        return Err(GhbareError::WorktreeError(format!(
            "git worktree add failed for branch '{}'",
            branch_name
        )));
    }

    Ok(())
}

fn run_post_add_commands(commands: &str, working_dir: &Path) -> Result<(), GhbareError> {
    if commands.trim().is_empty() {
        return Ok(());
    }
    eprintln!("Running post-add commands...");
    let status = Command::new("sh")
        .arg("-c")
        .arg(commands)
        .current_dir(working_dir)
        .status()
        .map_err(|e| GhbareError::WorktreeError(format!("Failed to execute: {}", e)))?;
    if !status.success() {
        return Err(GhbareError::WorktreeError(
            "Post-add commands failed".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_to_dirname() {
        assert_eq!(branch_to_dirname("feature/000"), "feature-000");
        assert_eq!(branch_to_dirname("fix/bug-123"), "fix-bug-123");
        assert_eq!(branch_to_dirname("main"), "main");
        assert_eq!(
            branch_to_dirname("feature/sub/deep"),
            "feature-sub-deep"
        );
    }

    #[test]
    fn test_generate_wip_branch_name() {
        let name = generate_wip_branch_name();
        assert!(name.starts_with("wip/"));
        // フォーマット確認: wip/MMDD-HHmmss
        let timestamp = name.strip_prefix("wip/").unwrap();
        let parts: Vec<&str> = timestamp.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 4); // MMDD
        assert_eq!(parts[1].len(), 6); // HHmmss
    }
}
