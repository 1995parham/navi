use crate::display;
use crate::env_var;
use crate::finder;
use crate::prelude::*;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use clap::Args;
use crossterm::style::Stylize;
use crossterm::style::style;
use serde::Deserialize;
use std::iter;
use std::process;

#[derive(Debug, Clone, Deserialize)]
struct PreviewContext {
    snippet: String,
    tags: String,
    comment: String,
    column: Option<String>,
    delimiter: Option<String>,
    map: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct Input {
    /// Selection line
    pub selection: String,
    /// Query match
    pub query: String,
    /// Typed text
    pub variable: String,
    /// Base64-encoded preview context (snippet, tags, comment, etc.)
    pub context: Option<String>,
}

impl Runnable for Input {
    fn run(&self) -> Result<()> {
        let selection = &self.selection;
        let query = &self.query;
        let variable = &self.variable;

        // Decode context from base64-encoded argument or fall back to environment variables
        let context = if let Some(ref encoded) = self.context {
            let decoded = BASE64
                .decode(encoded.as_bytes())
                .context("Failed to decode base64 context")?;
            let json_str =
                String::from_utf8(decoded).context("Failed to parse context as UTF-8")?;
            serde_json::from_str::<PreviewContext>(&json_str)
                .context("Failed to parse context JSON")?
        } else {
            // Fallback to environment variables for backward compatibility
            PreviewContext {
                snippet: env_var::must_get(env_var::PREVIEW_INITIAL_SNIPPET),
                tags: env_var::must_get(env_var::PREVIEW_TAGS),
                comment: env_var::must_get(env_var::PREVIEW_COMMENT),
                column: env_var::parse::<u8>(env_var::PREVIEW_COLUMN).map(|c| c.to_string()),
                delimiter: env_var::get(env_var::PREVIEW_DELIMITER).ok(),
                map: env_var::get(env_var::PREVIEW_MAP).ok(),
            }
        };

        let snippet = context.snippet;
        let tags = context.tags;
        let comment = context.comment;
        let column = context.column.as_ref().and_then(|c| c.parse::<u8>().ok());
        let delimiter = context.delimiter;
        let map = context.map;

        let active_color = CONFIG.tag_color();
        let inactive_color = CONFIG.comment_color();

        let mut colored_snippet = String::from(&snippet);
        let mut visited_vars: HashSet<&str> = HashSet::new();

        let mut variables = String::from("");

        println!(
            "{comment} {tags}",
            comment = style(comment).with(CONFIG.comment_color()),
            tags = style(format!("[{tags}]")).with(CONFIG.tag_color()),
        );

        let bracketed_current_variable = format!("<{variable}>");

        let bracketed_variables: Vec<&str> = {
            if snippet.contains(&bracketed_current_variable) {
                display::VAR_REGEX
                    .find_iter(&snippet)
                    .map(|m| m.as_str())
                    .collect()
            } else {
                iter::once(&bracketed_current_variable)
                    .map(|s| s.as_str())
                    .collect()
            }
        };

        for bracketed_variable_name in bracketed_variables {
            let variable_name = &bracketed_variable_name[1..bracketed_variable_name.len() - 1];

            if visited_vars.contains(variable_name) {
                continue;
            } else {
                visited_vars.insert(variable_name);
            }

            let is_current = variable_name == variable;
            let variable_color = if is_current {
                active_color
            } else {
                inactive_color
            };
            let env_variable_name = env_var::escape(variable_name);

            let value = if is_current {
                let v = selection.trim_matches('\'');
                if v.is_empty() {
                    query.trim_matches('\'')
                } else {
                    v
                }
                .to_string()
            } else if let Ok(v) = env_var::get(&env_variable_name) {
                v
            } else {
                "".to_string()
            };

            let replacement = format!(
                "{variable}",
                variable = style(bracketed_variable_name).with(variable_color),
            );

            colored_snippet = colored_snippet.replace(bracketed_variable_name, &replacement);

            variables = format!(
                "{variables}\n{variable} = {value}",
                variables = variables,
                variable = style(variable_name).with(variable_color),
                value = if env_var::get(&env_variable_name).is_ok() {
                    value
                } else if is_current {
                    finder::process(value, column, delimiter.as_deref(), map.clone())
                        .expect("Unable to process value")
                } else {
                    "".to_string()
                }
            );
        }

        println!(
            "{snippet}",
            snippet = display::fix_newlines(&colored_snippet)
        );
        println!("{variables}");

        process::exit(0)
    }
}
