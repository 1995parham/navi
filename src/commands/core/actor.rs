use crate::common::clipboard;
use crate::common::fs;
use crate::common::shell;
use crate::common::shell::ShellSpawnError;
use crate::common::types::{EnvVars, VariableCache};
use crate::config::Action;
use crate::display;
use crate::env_var;
use crate::finder::structures::{Opts as FinderOpts, SuggestionType};
use crate::prelude::*;
use crate::structures::cheat::{Suggestion, VariableMap};
use crate::structures::item::Item;
use std::io::Write as _;

use super::preview;
use super::suggestion;

fn prompt_finder(
    variable_name: &str,
    suggestion_option: Option<&Suggestion>,
    variable_count: usize,
    preview_context_env_vars: &EnvVars,
    variable_cache: &VariableCache,
) -> Result<String> {
    let mut preview_env_vars = preview_context_env_vars.clone();

    // Execute suggestion command and get options
    let (suggestions_text, finder_opts) = if let Some((command, opts)) = suggestion_option {
        // Apply suggestion options to preview environment variables
        let _extra_preview = opts
            .as_ref()
            .and_then(|o| suggestion::apply_suggestion_options(&mut preview_env_vars, o));

        let text = suggestion::execute_suggestion_command(command, variable_cache)?;
        (text, opts)
    } else {
        ("\n".to_string(), &None)
    };

    // Build shell-specific preview command
    let extra_preview = finder_opts.as_ref().and_then(|opts| opts.preview.as_ref());
    let preview_command =
        preview::build_preview_command(variable_name, extra_preview, &CONFIG.shell());

    // Build finder options
    let mut opts = FinderOpts {
        preview: Some(preview_command),
        show_all_columns: true,
        env_vars: preview_env_vars,
        ..finder_opts.clone().unwrap_or_else(FinderOpts::var_default)
    };

    // Apply variable-specific query and filter
    opts.query = env_var::get(format!("{variable_name}__query")).ok();

    if let Ok(filter) = env_var::get(format!("{variable_name}__best")) {
        opts.filter = Some(filter);
        opts.suggestion_type = SuggestionType::SingleSelection;
    }

    // Set preview window layout
    if opts.preview_window.is_none() {
        opts.preview_window = Some(preview::calculate_preview_window(
            extra_preview,
            variable_count,
        ));
    }

    // Disable suggestions if none provided
    if suggestion_option.is_none() {
        opts.suggestion_type = SuggestionType::Disabled;
    }

    // Call finder with suggestions
    let (output, _) = CONFIG
        .finder()
        .call(opts, |stdin| {
            stdin
                .write_all(suggestions_text.as_bytes())
                .context("Could not write to finder's stdin")?;
            Ok(())
        })
        .context("Finder was unable to prompt with suggestions")?;

    Ok(output)
}

fn unique_result_count(results: &[&str]) -> usize {
    let mut vars = results.to_owned();
    vars.sort_unstable();
    vars.dedup();
    vars.len()
}

fn replace_variables_from_snippet(
    snippet: &str,
    tags: &str,
    variable_map: VariableMap,
    preview_context_env_vars: &EnvVars,
) -> Result<String> {
    let mut interpolated_snippet = String::from(snippet);
    let mut variable_cache = VariableCache::new();

    if CONFIG.prevent_interpolation() {
        return Ok(interpolated_snippet);
    }

    // Find all variable references in the snippet (e.g., <variable_name>)
    let variable_references: Vec<&str> = display::VAR_REGEX
        .find_iter(snippet)
        .map(|m| m.as_str())
        .collect();
    let variable_count = unique_result_count(&variable_references);

    // Process each variable reference
    for variable_ref in variable_references {
        // Extract variable name from brackets: <name> -> name
        let variable_name = &variable_ref[1..variable_ref.len() - 1];
        let env_variable_name = env_var::escape(variable_name);

        // Get value from cache or prompt user
        let value = if let Some(cached) = variable_cache.get(&env_variable_name) {
            // Use cached value if available
            cached.clone()
        } else if let Some(suggestion) = variable_map.get_suggestion(tags, variable_name) {
            // Process suggestion with nested variable replacement
            let mut processed_suggestion = suggestion.clone();
            processed_suggestion.0 = replace_variables_from_snippet(
                &processed_suggestion.0,
                tags,
                variable_map.clone(),
                preview_context_env_vars,
            )?;

            // Prompt user with the processed suggestion
            prompt_finder(
                variable_name,
                Some(&processed_suggestion),
                variable_count,
                preview_context_env_vars,
                &variable_cache,
            )?
        } else {
            // No suggestion available, prompt user directly
            prompt_finder(
                variable_name,
                None,
                variable_count,
                preview_context_env_vars,
                &variable_cache,
            )?
        };

        // Cache the value for future references
        variable_cache.insert(env_variable_name, value.clone());

        // Replace variable reference in snippet
        interpolated_snippet = if value.as_str() == "\n" {
            // Empty value - remove the variable reference entirely
            interpolated_snippet.replacen(variable_ref, "", 1)
        } else {
            interpolated_snippet.replacen(variable_ref, value.as_str(), 1)
        };
    }

    Ok(interpolated_snippet)
}

pub fn with_absolute_path(snippet: String) -> String {
    if let Some(s) = snippet.strip_prefix("navi ") {
        return format!("{} {}", fs::exe_string(), s);
    }
    snippet
}

pub fn act(
    extractions: Result<(&str, Item)>,
    files: Vec<String>,
    variable_map: Option<VariableMap>,
) -> Result<()> {
    let (
        key,
        Item {
            tags,
            comment,
            snippet,
            file_index,
            ..
        },
    ) = extractions.unwrap();

    // Handle file editing shortcut
    if key == "ctrl-o" {
        edit::edit_file(Path::new(&files[file_index.expect("No files found")]))
            .expect("Could not open file in external editor");
        return Ok(());
    }

    // Build preview context for variable replacement
    let mut preview_context_env_vars = EnvVars::new();
    preview_context_env_vars.insert(
        env_var::PREVIEW_INITIAL_SNIPPET.to_string(),
        snippet.clone(),
    );
    preview_context_env_vars.insert(env_var::PREVIEW_TAGS.to_string(), tags.clone());
    preview_context_env_vars.insert(env_var::PREVIEW_COMMENT.to_string(), comment.to_string());

    // Process snippet: replace variables, convert paths, handle newlines
    let interpolated_snippet = {
        let mut s = replace_variables_from_snippet(
            &snippet,
            &tags,
            variable_map.expect("No variables received from finder"),
            &preview_context_env_vars,
        )
        .context("Failed to replace variables from snippet")?;
        s = with_absolute_path(s);
        s = display::with_new_lines(s);
        s
    };

    // Handle command editing shortcut
    if key == "ctrl-e" {
        // Create a temporary file with the snippet
        let mut temp_file = tempfile::Builder::new()
            .prefix("navi-")
            .suffix(".sh")
            .tempfile()
            .context("Failed to create temporary file")?;

        // Write the interpolated snippet to the temp file
        temp_file
            .write_all(interpolated_snippet.as_bytes())
            .context("Failed to write snippet to temporary file")?;

        // Get the path before the file is closed
        let temp_path = temp_file.path().to_path_buf();

        // Open the file in the user's EDITOR
        edit::edit_file(&temp_path)
            .context("Failed to open snippet in editor")?;

        // Read back the edited content
        let edited_snippet = std::fs::read_to_string(&temp_path)
            .context("Failed to read edited snippet")?;

        // Output the edited snippet for the user to execute
        println!("{}", edited_snippet.trim_end());

        return Ok(());
    }

    match CONFIG.action() {
        Action::Print => {
            println!("{interpolated_snippet}");
        }
        Action::Execute => match key {
            "ctrl-y" => {
                clipboard::copy(interpolated_snippet)?;
            }
            _ => {
                let mut cmd = shell::out();
                cmd.arg(&interpolated_snippet[..]);
                debug!(cmd = ?cmd);
                cmd.spawn()
                    .map_err(|e| ShellSpawnError::new(&interpolated_snippet[..], e))?
                    .wait()
                    .context("bash was not running")?;
            }
        },
    };

    Ok(())
}
