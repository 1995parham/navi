/// Helper module for executing suggestion commands
use crate::common::shell::{self, ShellSpawnError};
use crate::common::types::VariableCache;
use crate::env_var;
use crate::finder::structures::Opts as FinderOpts;
use crate::prelude::*;
use std::process::Stdio;

/// Execute a suggestion command and return the output
pub fn execute_suggestion_command(
    command: &str,
    variable_cache: &VariableCache,
) -> Result<String> {
    let mut cmd = shell::out();
    cmd.stdout(Stdio::piped())
        .arg(command)
        .envs(variable_cache);

    debug!(cmd = ?cmd);

    let child = cmd
        .spawn()
        .map_err(|e| ShellSpawnError::new(command, e))?;

    let output = child
        .wait_with_output()
        .context("Failed to wait and collect output from shell command")?;

    String::from_utf8(output.stdout)
        .context("Suggestion command output is not valid UTF-8")
}

/// Apply suggestion options to preview environment variables
pub fn apply_suggestion_options(
    preview_env_vars: &mut std::collections::HashMap<String, String>,
    options: &FinderOpts,
) -> Option<String> {
    let mut extra_preview = None;

    if let Some(column) = options.column {
        preview_env_vars.insert(
            env_var::PREVIEW_COLUMN.to_string(),
            column.to_string(),
        );
    }

    if let Some(ref delimiter) = options.delimiter {
        preview_env_vars.insert(
            env_var::PREVIEW_DELIMITER.to_string(),
            delimiter.clone(),
        );
    }

    if let Some(ref map) = options.map {
        preview_env_vars.insert(
            env_var::PREVIEW_MAP.to_string(),
            map.clone(),
        );
    }

    if let Some(ref preview) = options.preview {
        extra_preview = Some(preview.clone());
    }

    extra_preview
}
