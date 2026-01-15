use crate::error::GhbareError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoInfo {
    pub host: String,
    pub owner: String,
    pub repo: String,
}

impl RepoInfo {
    pub fn to_ssh_url(&self) -> String {
        format!("git@{}:{}/{}.git", self.host, self.owner, self.repo)
    }

    pub fn to_https_url(&self) -> String {
        format!("https://{}/{}/{}.git", self.host, self.owner, self.repo)
    }

    pub fn to_local_path(&self) -> String {
        format!("{}/{}/{}", self.host, self.owner, self.repo)
    }
}

pub fn parse_repo_url(input: &str) -> Result<RepoInfo, GhbareError> {
    let input = input.trim();

    if input.starts_with("git@") {
        return parse_ssh_url(input);
    }

    if input.starts_with("https://") || input.starts_with("http://") {
        return parse_https_url(input);
    }

    if input.starts_with("ssh://") {
        return parse_ssh_protocol_url(input);
    }

    parse_short_url(input)
}

fn parse_ssh_url(input: &str) -> Result<RepoInfo, GhbareError> {
    let without_prefix = input
        .strip_prefix("git@")
        .ok_or_else(|| GhbareError::UrlParseError(input.to_string()))?;

    let parts: Vec<&str> = without_prefix.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(GhbareError::UrlParseError(input.to_string()));
    }

    let host = parts[0].to_string();
    let path = parts[1].trim_end_matches(".git");

    parse_owner_repo(path, &host, input)
}

fn parse_https_url(input: &str) -> Result<RepoInfo, GhbareError> {
    let parsed =
        url::Url::parse(input).map_err(|_| GhbareError::UrlParseError(input.to_string()))?;

    let host = parsed
        .host_str()
        .ok_or_else(|| GhbareError::UrlParseError(input.to_string()))?
        .to_string();

    let path = parsed
        .path()
        .trim_start_matches('/')
        .trim_end_matches(".git");

    parse_owner_repo(path, &host, input)
}

fn parse_ssh_protocol_url(input: &str) -> Result<RepoInfo, GhbareError> {
    let parsed =
        url::Url::parse(input).map_err(|_| GhbareError::UrlParseError(input.to_string()))?;

    let host = parsed
        .host_str()
        .ok_or_else(|| GhbareError::UrlParseError(input.to_string()))?
        .to_string();

    let path = parsed
        .path()
        .trim_start_matches('/')
        .trim_end_matches(".git");

    parse_owner_repo(path, &host, input)
}

fn parse_short_url(input: &str) -> Result<RepoInfo, GhbareError> {
    let path = input.trim_end_matches(".git");
    let parts: Vec<&str> = path.splitn(3, '/').collect();

    if parts.len() != 3 {
        return Err(GhbareError::UrlParseError(input.to_string()));
    }

    Ok(RepoInfo {
        host: parts[0].to_string(),
        owner: parts[1].to_string(),
        repo: parts[2].to_string(),
    })
}

fn parse_owner_repo(path: &str, host: &str, original: &str) -> Result<RepoInfo, GhbareError> {
    let parts: Vec<&str> = path.splitn(2, '/').collect();

    if parts.len() != 2 {
        return Err(GhbareError::UrlParseError(original.to_string()));
    }

    Ok(RepoInfo {
        host: host.to_string(),
        owner: parts[0].to_string(),
        repo: parts[1].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_short_url() {
        let info = parse_repo_url("github.com/user/repo").unwrap();
        assert_eq!(info.host, "github.com");
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_parse_short_url_with_git_suffix() {
        let info = parse_repo_url("github.com/user/repo.git").unwrap();
        assert_eq!(info.host, "github.com");
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_parse_ssh_url() {
        let info = parse_repo_url("git@github.com:user/repo.git").unwrap();
        assert_eq!(info.host, "github.com");
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_parse_https_url() {
        let info = parse_repo_url("https://github.com/user/repo").unwrap();
        assert_eq!(info.host, "github.com");
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_parse_https_url_with_git_suffix() {
        let info = parse_repo_url("https://github.com/user/repo.git").unwrap();
        assert_eq!(info.host, "github.com");
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_to_ssh_url() {
        let info = RepoInfo {
            host: "github.com".to_string(),
            owner: "user".to_string(),
            repo: "repo".to_string(),
        };
        assert_eq!(info.to_ssh_url(), "git@github.com:user/repo.git");
    }

    #[test]
    fn test_to_https_url() {
        let info = RepoInfo {
            host: "github.com".to_string(),
            owner: "user".to_string(),
            repo: "repo".to_string(),
        };
        assert_eq!(info.to_https_url(), "https://github.com/user/repo.git");
    }

    #[test]
    fn test_to_local_path() {
        let info = RepoInfo {
            host: "github.com".to_string(),
            owner: "user".to_string(),
            repo: "repo".to_string(),
        };
        assert_eq!(info.to_local_path(), "github.com/user/repo");
    }

    #[test]
    fn test_invalid_url() {
        let result = parse_repo_url("invalid");
        assert!(result.is_err());
    }
}
