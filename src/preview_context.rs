use crate::env_var;
use std::collections::HashMap;

/// Context information passed to preview windows via environment variables.
/// This is used for inter-process communication between navi and skim preview windows.
///
/// Note: This type is currently not used but provides a typed interface for future improvements.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PreviewContext {
    /// The original snippet with variables (e.g., "git checkout <branch>")
    pub snippet: String,
    /// Comma-separated tags for the snippet
    pub tags: String,
    /// Human-readable description of the snippet
    pub comment: String,
    /// Optional: which column to extract from multi-column output
    pub column: Option<u8>,
    /// Optional: delimiter for parsing columns
    pub delimiter: Option<String>,
    /// Optional: transformation/mapping command for the value
    pub map: Option<String>,
}

#[allow(dead_code)]
impl PreviewContext {
    pub fn new(snippet: String, tags: String, comment: String) -> Self {
        Self {
            snippet,
            tags,
            comment,
            column: None,
            delimiter: None,
            map: None,
        }
    }

    /// Convert this context to environment variables for the preview subprocess
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        vars.insert(
            env_var::PREVIEW_INITIAL_SNIPPET.to_string(),
            self.snippet.clone(),
        );
        vars.insert(env_var::PREVIEW_TAGS.to_string(), self.tags.clone());
        vars.insert(env_var::PREVIEW_COMMENT.to_string(), self.comment.clone());

        if let Some(col) = self.column {
            vars.insert(env_var::PREVIEW_COLUMN.to_string(), col.to_string());
        }
        if let Some(ref delim) = self.delimiter {
            vars.insert(env_var::PREVIEW_DELIMITER.to_string(), delim.clone());
        }
        if let Some(ref m) = self.map {
            vars.insert(env_var::PREVIEW_MAP.to_string(), m.clone());
        }

        vars
    }

    /// Add column extraction configuration
    pub fn with_column(mut self, column: Option<u8>) -> Self {
        self.column = column;
        self
    }

    /// Add delimiter configuration
    pub fn with_delimiter(mut self, delimiter: Option<String>) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Add map configuration
    pub fn with_map(mut self, map: Option<String>) -> Self {
        self.map = map;
        self
    }
}
