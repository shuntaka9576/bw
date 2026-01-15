use crate::error::GhbareError;
use git2::{FetchOptions, RemoteCallbacks, Repository};
use std::path::Path;

pub fn bare_clone(url: &str, dest: &Path) -> Result<Repository, GhbareError> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|_url, username_from_url, allowed_types| {
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        } else if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            let username = std::env::var("GIT_USERNAME").unwrap_or_default();
            let password = std::env::var("GIT_PASSWORD").unwrap_or_default();
            git2::Cred::userpass_plaintext(&username, &password)
        } else {
            Err(git2::Error::from_str("no authentication available"))
        }
    });

    callbacks.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            eprint!(
                "\rResolving deltas {}/{}   ",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            eprint!(
                "\rReceiving objects: {:3}% ({}/{})   ",
                100 * stats.received_objects() / stats.total_objects(),
                stats.received_objects(),
                stats.total_objects()
            );
        }
        true
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.bare(true);
    builder.fetch_options(fetch_options);

    let repo = builder
        .clone(url, dest)
        .map_err(|e| GhbareError::CloneError(e.message().to_string()))?;

    eprintln!();

    Ok(repo)
}
