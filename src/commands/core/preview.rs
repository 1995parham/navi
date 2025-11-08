/// Helper module for building shell-specific preview commands
use crate::common::fs;
use crate::common::shell::EOF;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

/// Constants for preview window layout
mod constants {
    pub const DEFAULT_PREVIEW_DIRECTION: &str = "up";
    pub const EXTRA_PREVIEW_DIRECTION: &str = "right:50%";
}

/// Preview context data that needs to be passed to preview commands
pub struct PreviewContext<'a> {
    pub snippet: &'a str,
    pub tags: &'a str,
    pub comment: &'a str,
    pub column: Option<&'a str>,
    pub delimiter: Option<&'a str>,
    pub map: Option<&'a str>,
}

impl<'a> PreviewContext<'a> {
    /// Encode context as base64 for safe passing through shell
    fn encode(&self) -> String {
        let json = serde_json::json!({
            "snippet": self.snippet,
            "tags": self.tags,
            "comment": self.comment,
            "column": self.column,
            "delimiter": self.delimiter,
            "map": self.map,
        });
        BASE64.encode(json.to_string().as_bytes())
    }
}

pub fn build_preview_command(
    variable_name: &str,
    extra_preview: Option<&String>,
    shell: &str,
    context: &PreviewContext,
) -> String {
    let exe = fs::exe_string();
    let extra = format_extra_preview(extra_preview);
    let context_encoded = context.encode();

    if shell.contains("powershell") {
        build_powershell_preview(&exe, variable_name, &extra, &context_encoded)
    } else if shell.contains("cmd.exe") {
        build_cmd_preview(&exe, variable_name, extra_preview, &context_encoded)
    } else if shell.contains("fish") {
        build_fish_preview(&exe, variable_name, &extra, &context_encoded)
    } else {
        build_unix_preview(&exe, variable_name, &extra, &context_encoded)
    }
}

pub fn calculate_preview_window(extra_preview: Option<&String>, variable_count: usize) -> String {
    if extra_preview.is_none() {
        format!(
            "{}:{}",
            constants::DEFAULT_PREVIEW_DIRECTION,
            variable_count + 3
        )
    } else {
        constants::EXTRA_PREVIEW_DIRECTION.to_string()
    }
}

fn format_extra_preview(extra: Option<&String>) -> String {
    extra.map(|e| format!(" echo; {e}")).unwrap_or_default()
}

fn build_powershell_preview(exe: &str, name: &str, extra: &str, context: &str) -> String {
    format!(
        r#"{exe} preview-var {{+}} "{{q}}" "{name}" "{context}"; {extra}"#,
        exe = exe,
        name = name,
        context = context,
        extra = extra,
    )
}

fn build_cmd_preview(exe: &str, name: &str, extra: Option<&String>, context: &str) -> String {
    format!(
        r#"(@echo.{{+}}{eof}{{q}}{eof}{name}{eof}{context}{eof}{extra}) | {exe} preview-var-stdin"#,
        exe = exe,
        name = name,
        context = context,
        extra = extra.cloned().unwrap_or_default(),
        eof = EOF,
    )
}

fn build_fish_preview(exe: &str, name: &str, extra: &str, context: &str) -> String {
    format!(
        r#"{exe} preview-var "{{+}}" "{{q}}" "{name}" "{context}"; {extra}"#,
        exe = exe,
        name = name,
        context = context,
        extra = extra,
    )
}

fn build_unix_preview(exe: &str, name: &str, extra: &str, context: &str) -> String {
    format!(
        r#"{exe} preview-var "$(cat <<{eof}
{{+}}
{eof}
)" "$(cat <<{eof}
{{q}}
{eof}
)" "{name}" "{context}"; {extra}"#,
        exe = exe,
        name = name,
        context = context,
        extra = extra,
        eof = EOF,
    )
}
