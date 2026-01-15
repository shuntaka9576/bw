use thiserror::Error;

#[derive(Debug, Error)]
pub enum GhbareError {
    #[error("Failed to parse repository URL: {0}")]
    UrlParseError(String),

    #[error("Config not found: {0}")]
    ConfigNotFound(String),

    #[error("Failed to parse config: {0}")]
    ConfigParseError(String),

    #[error("$EDITOR environment variable is not set")]
    EditorNotFound,

    #[error("Clone failed: {0}")]
    CloneError(String),

    #[error("Post clone command failed: {0}")]
    PostCloneCommandError(String),

    #[error("Repository already exists: {0}")]
    RepositoryAlreadyExists(String),

    #[error("Repository root not found (no .bare directory)")]
    RepoRootNotFound,

    #[error("Worktree operation failed: {0}")]
    WorktreeError(String),

    #[error("Worktree already exists: {0}")]
    WorktreeAlreadyExists(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
