use crate::prelude::*;
use clap::ValueEnum;
use std::process::Command;
use thiserror::Error;

pub const EOF: &str = "NAVIEOF";

/// Picks a heredoc terminator that does not occur as a line of `text`.
///
/// A heredoc ends at the first line equal to its terminator, so splicing text
/// containing a bare `NAVIEOF` line into `<<'NAVIEOF'` would close the heredoc
/// early and let the remainder of that text run as shell code. Extending the
/// terminator until it no longer collides keeps the surrounding script inert.
pub fn heredoc_delimiter(text: &str) -> String {
    let mut delimiter = String::from(EOF);
    while text.lines().any(|line| line.trim() == delimiter) {
        delimiter.push('_');
    }
    delimiter
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Elvish,
    Nushell,
    Powershell,
}

#[derive(Error, Debug)]
#[error("Failed to spawn child process `bash` to execute `{command}`")]
pub struct ShellSpawnError {
    command: String,
    #[source]
    source: anyhow::Error,
}

impl ShellSpawnError {
    pub fn new<SourceError>(command: impl Into<String>, source: SourceError) -> Self
    where
        SourceError: std::error::Error + Sync + Send + 'static,
    {
        ShellSpawnError {
            command: command.into(),
            source: source.into(),
        }
    }
}

pub fn out() -> Result<Command> {
    let words_str = CONFIG.shell();
    let mut words_vec = shellwords::split(&words_str).context("Failed to parse shell command")?;
    let mut words = words_vec.iter_mut();
    let first_cmd = words
        .next()
        .ok_or_else(|| anyhow!("Shell command is empty"))?;
    let mut cmd = Command::new(first_cmd);
    cmd.args(words);
    let dash_c = if words_str.contains("cmd.exe") {
        "/c"
    } else {
        "-c"
    };
    cmd.arg(dash_c);
    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heredoc_delimiter_is_unchanged_for_ordinary_text() {
        assert_eq!(heredoc_delimiter("git status"), "NAVIEOF");
        assert_eq!(heredoc_delimiter(""), "NAVIEOF");
        // A mention that is not a line of its own cannot terminate a heredoc.
        assert_eq!(heredoc_delimiter("echo NAVIEOF please"), "NAVIEOF");
    }

    #[test]
    fn test_heredoc_delimiter_avoids_collisions() {
        assert_eq!(heredoc_delimiter("a\nNAVIEOF\nb"), "NAVIEOF_");
        assert_eq!(heredoc_delimiter("a\nNAVIEOF\nNAVIEOF_\nb"), "NAVIEOF__");
        // Surrounding whitespace is treated as a collision rather than trusted.
        assert_eq!(heredoc_delimiter("a\n  NAVIEOF  \nb"), "NAVIEOF_");
    }

    #[test]
    fn test_heredoc_delimiter_never_appears_in_text() {
        for text in ["plain", "a\nNAVIEOF\nb", "NAVIEOF\nNAVIEOF_\nNAVIEOF__"] {
            let d = heredoc_delimiter(text);
            assert!(
                !text.lines().any(|l| l.trim() == d),
                "delimiter {d:?} collides with {text:?}"
            );
        }
    }
}
