/// Helper module for building shell-specific preview commands
use crate::common::fs;
use crate::common::shell::EOF;

/// Constants for preview window layout
mod constants {
    pub const DEFAULT_PREVIEW_DIRECTION: &str = "up";
    pub const EXTRA_PREVIEW_DIRECTION: &str = "right:50%";
}

pub fn build_preview_command(
    variable_name: &str,
    extra_preview: Option<&String>,
    shell: &str,
) -> String {
    let exe = fs::exe_string();
    let extra = format_extra_preview(extra_preview);

    if shell.contains("powershell") {
        build_powershell_preview(&exe, variable_name, &extra)
    } else if shell.contains("cmd.exe") {
        build_cmd_preview(&exe, variable_name, extra_preview)
    } else if shell.contains("fish") {
        build_fish_preview(&exe, variable_name, &extra)
    } else {
        build_unix_preview(&exe, variable_name, &extra)
    }
}

pub fn calculate_preview_window(
    extra_preview: Option<&String>,
    variable_count: usize,
) -> String {
    if extra_preview.is_none() {
        format!("{}:{}", constants::DEFAULT_PREVIEW_DIRECTION, variable_count + 3)
    } else {
        constants::EXTRA_PREVIEW_DIRECTION.to_string()
    }
}

fn format_extra_preview(extra: Option<&String>) -> String {
    extra
        .map(|e| format!(" echo; {e}"))
        .unwrap_or_default()
}

fn build_powershell_preview(exe: &str, name: &str, extra: &str) -> String {
    format!(
        r#"{exe} preview-var {{+}} "{{q}}" "{name}"; {extra}"#,
        exe = exe,
        name = name,
        extra = extra,
    )
}

fn build_cmd_preview(exe: &str, name: &str, extra: Option<&String>) -> String {
    format!(
        r#"(@echo.{{+}}{eof}{{q}}{eof}{name}{eof}{extra}) | {exe} preview-var-stdin"#,
        exe = exe,
        name = name,
        extra = extra.cloned().unwrap_or_default(),
        eof = EOF,
    )
}

fn build_fish_preview(exe: &str, name: &str, extra: &str) -> String {
    format!(
        r#"{exe} preview-var "{{+}}" "{{q}}" "{name}"; {extra}"#,
        exe = exe,
        name = name,
        extra = extra,
    )
}

fn build_unix_preview(exe: &str, name: &str, extra: &str) -> String {
    format!(
        r#"{exe} preview-var "$(cat <<{eof}
{{+}}
{eof}
)" "$(cat <<{eof}
{{q}}
{eof}
)" "{name}"; {extra}"#,
        exe = exe,
        name = name,
        extra = extra,
        eof = EOF,
    )
}
