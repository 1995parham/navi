use crate::common::shell::ShellSpawnError;
use crate::prelude::*;
use std::process::Command;

pub fn shallow_clone(uri: &str, target: &str) -> Result<()> {
    Command::new("git")
        .args(["clone", uri, target, "--depth", "1"])
        .spawn()
        .map_err(|e| ShellSpawnError::new("git clone", e))?
        .wait()
        .context("Unable to git clone")?;
    Ok(())
}

/// Extracts `(uri, user, repo)` from a repository URI, which may be a bare
/// `user/repo`, an HTTPS URL or an SSH remote.
pub fn meta(uri: &str) -> Result<(String, String, String)> {
    let actual_uri = if uri.contains("://") || uri.contains('@') {
        uri.to_string()
    } else {
        format!("https://github.com/{uri}")
    };

    // Turning `:` into `/` folds `git@host:user/repo` into the same shape as an
    // HTTPS URL. Empty segments are skipped so that `https://` and a trailing
    // slash don't count as path components.
    let uri_to_split = actual_uri.replace(':', "/");
    let mut parts = uri_to_split.rsplit('/').filter(|p| !p.is_empty());

    let repo = parts
        .next()
        .with_context(|| format!("Invalid repository URI `{uri}`: no repository name"))?;
    let user = parts
        .next()
        .with_context(|| format!("Invalid repository URI `{uri}`: expected a `user/repo` path"))?;

    // Only a trailing `.git` is a suffix; `user.github.io` must survive intact.
    let repo = repo.strip_suffix(".git").unwrap_or(repo);

    Ok((actual_uri, user.to_string(), repo.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_github_https() {
        let (actual_uri, user, repo) = meta("https://github.com/denisidoro/navi").unwrap();
        assert_eq!(actual_uri, "https://github.com/denisidoro/navi".to_string());
        assert_eq!(user, "denisidoro".to_string());
        assert_eq!(repo, "navi".to_string());
    }

    #[test]
    fn test_meta_github_ssh() {
        let (actual_uri, user, repo) = meta("git@github.com:denisidoro/navi.git").unwrap();
        assert_eq!(actual_uri, "git@github.com:denisidoro/navi.git".to_string());
        assert_eq!(user, "denisidoro".to_string());
        assert_eq!(repo, "navi".to_string());
    }

    #[test]
    fn test_meta_gitlab_https() {
        let (actual_uri, user, repo) = meta("https://gitlab.com/user/repo.git").unwrap();
        assert_eq!(actual_uri, "https://gitlab.com/user/repo.git".to_string());
        assert_eq!(user, "user".to_string());
        assert_eq!(repo, "repo".to_string());
    }

    #[test]
    fn test_meta_shorthand() {
        let (actual_uri, user, repo) = meta("denisidoro/navi").unwrap();
        assert_eq!(actual_uri, "https://github.com/denisidoro/navi".to_string());
        assert_eq!(user, "denisidoro".to_string());
        assert_eq!(repo, "navi".to_string());
    }

    #[test]
    fn test_meta_trailing_slash() {
        let (_, user, repo) = meta("https://github.com/denisidoro/navi/").unwrap();
        assert_eq!(user, "denisidoro".to_string());
        assert_eq!(repo, "navi".to_string());
    }

    #[test]
    fn test_meta_strips_only_a_trailing_dot_git() {
        let (_, user, repo) = meta("https://github.com/parham/parham.github.io").unwrap();
        assert_eq!(user, "parham".to_string());
        assert_eq!(repo, "parham.github.io".to_string());
    }

    #[test]
    fn test_meta_rejects_malformed_uris() {
        // These used to panic with a `usize` subtract-with-overflow.
        for uri in ["@", "user@host", "git@github.com"] {
            assert!(meta(uri).is_err(), "expected `{uri}` to be rejected");
        }
    }
}
