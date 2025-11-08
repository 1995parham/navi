use crate::common::fs;
use crate::display;
use crate::finder::structures::{Opts as FinderOpts, SuggestionType};
use crate::prelude::*;
use crate::structures::cheat::VariableMap;
use crate::structures::item::Item;
use std::env;
use std::io::Write;

use std::sync::LazyLock;

pub static VAR_LINE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\$\s*([^:]+):(.*)").expect("Invalid regex"));

fn parse_opts(text: &str) -> Result<FinderOpts> {
    let mut multi = false;
    let mut prevent_extra = false;

    let mut opts = FinderOpts::var_default();

    let parts = shellwords::split(text)
        .map_err(|_| anyhow!("Given options are missing a closing quote"))?;

    parts
        .into_iter()
        .filter(|part| {
            // We'll take parts in pairs of 2: (argument, value). Flags don't have a value tho, so we filter and handle them beforehand.
            match part.as_str() {
                "--multi" => {
                    multi = true;
                    false
                }
                "--prevent-extra" => {
                    prevent_extra = true;
                    false
                }
                "--expand" => {
                    opts.map = Some(format!("{} fn map::expand", fs::exe_string()));
                    false
                }
                _ => true,
            }
        })
        .collect::<Vec<_>>()
        .chunks(2)
        .try_for_each(|flag_and_value| {
            if let [flag, value] = flag_and_value {
                match flag.as_str() {
                    "--headers" | "--header-lines" => {
                        opts.header_lines = value
                            .parse::<u8>()
                            .context("Value for `--headers` is invalid u8")?
                    }
                    "--column" => {
                        opts.column = Some(
                            value
                                .parse::<u8>()
                                .context("Value for `--column` is invalid u8")?,
                        )
                    }
                    "--map" => opts.map = Some(value.to_string()),
                    "--delimiter" => opts.delimiter = Some(value.to_string()),
                    "--query" => opts.query = Some(value.to_string()),
                    "--filter" => opts.filter = Some(value.to_string()),
                    "--preview" => opts.preview = Some(value.to_string()),
                    "--preview-window" => opts.preview_window = Some(value.to_string()),
                    "--header" => opts.header = Some(value.to_string()),
                    "--fzf-overrides" => opts.overrides = Some(value.to_string()),
                    _ => (),
                }
                Ok(())
            } else if let [flag] = flag_and_value {
                Err(anyhow!("No value provided for the flag `{}`", flag))
            } else {
                unreachable!() // Chunking by 2 allows only for tuples of 1 or 2 items...
            }
        })
        .context("Failed to parse finder options")?;

    let suggestion_type = match (multi, prevent_extra) {
        (true, _) => SuggestionType::MultipleSelections, // multi wins over prevent-extra
        (false, false) => SuggestionType::SingleRecommendation,
        (false, true) => SuggestionType::SingleSelection,
    };
    opts.suggestion_type = suggestion_type;

    Ok(opts)
}

fn parse_variable_line(line: &str) -> Result<(&str, &str, Option<FinderOpts>)> {
    let caps = VAR_LINE_REGEX.captures(line).ok_or_else(|| {
        anyhow!(
            "No variables, command, and options found in the line `{}`",
            line
        )
    })?;
    let variable = caps
        .get(1)
        .ok_or_else(|| anyhow!("No variable captured in the line `{}`", line))?
        .as_str()
        .trim();
    let mut command_plus_opts = caps
        .get(2)
        .ok_or_else(|| anyhow!("No command and options captured in the line `{}`", line))?
        .as_str()
        .split("---");
    let command = command_plus_opts
        .next()
        .ok_or_else(|| anyhow!("No command captured in the line `{}`", line))?;
    let command_options = command_plus_opts.next().map(parse_opts).transpose()?;
    Ok((variable, command, command_options))
}

fn without_prefix(line: &str) -> String {
    if line.len() > 2 {
        String::from(line[2..].trim())
    } else {
        String::from("")
    }
}

#[derive(Clone, Default)]
pub struct FilterOpts {
    pub allowlist: Vec<String>,
    pub denylist: Vec<String>,
    pub hash: Option<u64>,
}

pub struct Parser<'a> {
    pub variables: VariableMap,
    visited_lines: HashSet<u64>,
    filter: FilterOpts,
    writer: &'a mut dyn Write,
    write_fn: fn(&Item) -> String,
}

fn without_first(string: &str) -> String {
    string
        .char_indices()
        .next()
        .and_then(|(i, _)| string.get(i + 1..))
        .expect("Should have at least one char")
        .to_string()
}

fn get_current_os() -> String {
    std::env::consts::OS.to_string()
}

fn matches_path_pattern(current_dir: &str, pattern: &str) -> bool {
    let pattern = pattern.trim();

    let pattern_regex = pattern
        .replace("**", "DOUBLE_STAR")
        .replace('*', "[^/]*")
        .replace("DOUBLE_STAR", ".*");

    let Ok(re) = Regex::new(&format!("^{}$", pattern_regex)) else {
        return false;
    };

    re.is_match(current_dir)
}

fn should_show_for_path(path_filter: &Option<String>) -> bool {
    let Some(filter) = path_filter else {
        return true;
    };

    let Ok(current_dir) = env::current_dir() else {
        return false;
    };

    filter
        .split(',')
        .map(str::trim)
        .any(|pattern| matches_path_pattern(&current_dir.to_string_lossy(), pattern))
}

fn should_show_for_os(os_filter: &Option<String>) -> bool {
    let Some(filter) = os_filter else {
        return true;
    };

    let current_os = get_current_os();

    for os_rule in filter.split(',').map(str::trim) {
        if let Some(excluded_os) = os_rule.strip_prefix('!') {
            if current_os == excluded_os {
                return false;
            }
        } else if current_os == os_rule {
            return true;
        }
    }

    !filter.split(',').any(|s| !s.trim().starts_with('!'))
}

fn get_current_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "localhost".to_string())
}

fn should_show_for_hostname(hostname_filter: &Option<String>) -> bool {
    let Some(filter) = hostname_filter else {
        return true;
    };

    let current_hostname = get_current_hostname();

    for hostname_rule in filter.split(',').map(str::trim) {
        if let Some(excluded_hostname) = hostname_rule.strip_prefix('!') {
            if current_hostname == excluded_hostname {
                return false;
            }
        } else if current_hostname == hostname_rule {
            return true;
        }
    }

    !filter.split(',').any(|s| !s.trim().starts_with('!'))
}

fn gen_lists(tag_rules: &str) -> FilterOpts {
    let words: Vec<_> = tag_rules.split(',').collect();

    let allowlist = words
        .iter()
        .filter(|w| !w.starts_with('!'))
        .map(|w| w.to_string())
        .collect();

    let denylist = words
        .iter()
        .filter(|w| w.starts_with('!'))
        .map(|w| without_first(w))
        .collect();

    FilterOpts {
        allowlist,
        denylist,
        ..Default::default()
    }
}

impl<'a> Parser<'a> {
    pub fn new(writer: &'a mut dyn Write, _is_terminal: bool) -> Self {
        let write_fn = display::terminal::write;

        let filter = match CONFIG.tag_rules() {
            Some(tr) => gen_lists(&tr),
            None => Default::default(),
        };

        Self {
            variables: Default::default(),
            visited_lines: Default::default(),
            filter,
            write_fn,
            writer,
        }
    }

    pub fn set_hash(&mut self, hash: u64) {
        self.filter.hash = Some(hash)
    }

    fn write_cmd(&mut self, item: &Item) -> Result<()> {
        if item.comment.is_empty() || item.snippet.trim().is_empty() {
            return Ok(());
        }

        let hash = item.hash();
        if self.visited_lines.contains(&hash) {
            return Ok(());
        }
        self.visited_lines.insert(hash);

        if !self.filter.denylist.is_empty() {
            for v in &self.filter.denylist {
                if item.tags.contains(v) {
                    return Ok(());
                }
            }
        }

        if !self.filter.allowlist.is_empty() {
            let mut should_allow = false;
            for v in &self.filter.allowlist {
                if item.tags.contains(v) {
                    should_allow = true;
                    break;
                }
            }
            if !should_allow {
                return Ok(());
            }
        }

        if let Some(h) = self.filter.hash
            && h != hash
        {
            return Ok(());
        }

        // Filter by path
        if !should_show_for_path(&item.path_filter) {
            return Ok(());
        }

        // Filter by OS
        if !should_show_for_os(&item.os_filter) {
            return Ok(());
        }

        // Filter by hostname
        if !should_show_for_hostname(&item.hostname_filter) {
            return Ok(());
        }

        let write_fn = self.write_fn;

        self.writer
            .write_all(write_fn(item).as_bytes())
            .context("Failed to write command to finder's stdin")
    }

    pub fn read_lines(
        &mut self,
        lines: impl Iterator<Item = Result<String>>,
        id: &str,
        file_index: Option<usize>,
    ) -> Result<()> {
        let mut item = Item::new(file_index);

        let mut should_break = false;

        let mut variable_cmd = String::from("");

        for (line_nr, line_result) in lines.enumerate() {
            let line = line_result.with_context(|| {
                format!("Failed to read line number {line_nr} in cheatsheet `{id}`")
            })?;

            if should_break {
                break;
            }

            // duplicate
            // if !item.tags.is_empty() && !item.comment.is_empty() {}

            // blank
            if line.is_empty() {
                if !item.snippet.is_empty() {
                    item.snippet.push_str(display::LINE_SEPARATOR);
                }
            }
            // tag
            else if line.starts_with('%') {
                should_break = self.write_cmd(&item).is_err();
                item.snippet = String::from("");
                item.tags = without_prefix(&line);
            }
            // dependency
            else if line.starts_with('@') {
                let tags_dependency = without_prefix(&line);
                self.variables
                    .insert_dependency(&item.tags, &tags_dependency);
            }
            // path filter
            else if let Some(path) = line.strip_prefix("; path:") {
                item.path_filter = Some(path.trim().into());
            }
            // os filter
            else if let Some(os) = line.strip_prefix("; os:") {
                item.os_filter = Some(os.trim().into());
            }
            // hostname filter
            else if let Some(hostname) = line.strip_prefix("; hostname:") {
                item.hostname_filter = Some(hostname.trim().into());
            }
            // metacomment
            else if line.starts_with(';') {
            }
            // comment
            else if line.starts_with('#') {
                should_break = self.write_cmd(&item).is_err();
                item.snippet = String::from("");
                item.comment = without_prefix(&line);
            }
            // variable
            else if !variable_cmd.is_empty() || (line.starts_with('$') && line.contains(':')) {
                should_break = self.write_cmd(&item).is_err();

                item.snippet = String::from("");

                variable_cmd.push_str(line.trim_end_matches('\\'));

                if !line.ends_with('\\') {
                    let full_variable_cmd = variable_cmd.clone();
                    let (variable, command, opts) =
                        parse_variable_line(&full_variable_cmd).with_context(|| {
                            format!(
                                "Failed to parse variable line. See line number {} in cheatsheet `{}`",
                                line_nr + 1,
                                id
                            )
                        })?;
                    variable_cmd = String::from("");
                    self.variables.insert_suggestion(
                        &item.tags,
                        variable,
                        (String::from(command), opts),
                    );
                }
            }
            // snippet
            else {
                if !item.snippet.is_empty() {
                    item.snippet.push_str(display::LINE_SEPARATOR);
                }
                item.snippet.push_str(&line);
            }
        }

        if !should_break {
            let _ = self.write_cmd(&item);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_variable_line() {
        let (variable, command, command_options) =
            parse_variable_line("$ user : echo -e \"$(whoami)\\nroot\" --- --prevent-extra")
                .unwrap();
        assert_eq!(command, " echo -e \"$(whoami)\\nroot\" ");
        assert_eq!(variable, "user");
        let opts = command_options.unwrap();
        assert_eq!(opts.header_lines, 0);
        assert_eq!(opts.column, None);
        assert_eq!(opts.delimiter, None);
        assert_eq!(opts.suggestion_type, SuggestionType::SingleSelection);
    }

    #[test]
    fn test_path_pattern_matching() {
        // Test exact match
        assert!(matches_path_pattern(
            "/home/user/projects",
            "/home/user/projects"
        ));

        // Test single star
        assert!(matches_path_pattern("/home/user/test", "/home/user/*"));
        assert!(matches_path_pattern("/home/user/projects", "/home/user/*"));
        assert!(!matches_path_pattern("/home/user/sub/dir", "/home/user/*"));

        // Test double star
        assert!(matches_path_pattern("/home/user/projects", "**/projects"));
        assert!(matches_path_pattern("/var/lib/projects", "**/projects"));
        assert!(matches_path_pattern(
            "/home/user/code/projects",
            "**/projects"
        ));
        assert!(matches_path_pattern(
            "/home/user/projects/sub",
            "**/projects/**"
        ));
        assert!(matches_path_pattern(
            "/home/user/projects/sub/deep",
            "**/projects/**"
        ));

        // Test wildcard in middle
        assert!(matches_path_pattern("/home/user/git-repo", "**/git-*"));
        assert!(matches_path_pattern(
            "/home/user/git-repo/src",
            "**/git-*/**"
        ));
        assert!(matches_path_pattern("/var/git-main/src", "**/git-*/**"));
        assert!(!matches_path_pattern("/home/user/svn-repo", "**/git-*/**"));
    }

    #[test]
    fn test_os_filtering() {
        let current_os = get_current_os();

        // No filter - should always show
        assert!(should_show_for_os(&None));

        // Positive match
        assert!(should_show_for_os(&Some(current_os.clone())));

        // Different OS - should not show
        let other_os = if current_os == "linux" {
            "windows"
        } else {
            "linux"
        };
        assert!(!should_show_for_os(&Some(other_os.to_string())));

        // Negation - exclude current OS
        assert!(!should_show_for_os(&Some(format!("!{}", current_os))));

        // Negation - exclude different OS (should show)
        assert!(should_show_for_os(&Some(format!("!{}", other_os))));

        // Multiple values with current OS
        assert!(should_show_for_os(&Some(format!(
            "{}, windows, macos",
            current_os
        ))));

        // Multiple values without current OS
        let filter = if current_os == "linux" {
            "windows, macos"
        } else {
            "linux"
        };
        assert!(!should_show_for_os(&Some(filter.to_string())));
    }

    #[test]
    fn test_path_filtering() {
        // No filter - should always show
        assert!(should_show_for_path(&None));

        // With filter - depends on current directory
        // We can't test the actual path matching without knowing the test runner's pwd,
        // but we can verify the function doesn't panic
        let _ = should_show_for_path(&Some("**/projects/**".to_string()));
        let _ = should_show_for_path(&Some("/home/user/*, /var/**".to_string()));
    }

    #[test]
    fn test_hostname_filtering() {
        let current_hostname = get_current_hostname();

        // No filter - should always show
        assert!(should_show_for_hostname(&None));

        // Positive match
        assert!(should_show_for_hostname(&Some(current_hostname.clone())));

        // Different hostname - should not show
        assert!(!should_show_for_hostname(&Some("other-host".to_string())));

        // Negation - exclude current hostname
        assert!(!should_show_for_hostname(&Some(format!(
            "!{}",
            current_hostname
        ))));

        // Negation - exclude different hostname (should show)
        assert!(should_show_for_hostname(&Some("!other-host".to_string())));

        // Multiple values with current hostname
        assert!(should_show_for_hostname(&Some(format!(
            "{}, server1, server2",
            current_hostname
        ))));

        // Multiple values without current hostname
        assert!(!should_show_for_hostname(&Some(
            "server1, server2".to_string()
        )));

        // Multiple negations excluding current hostname
        assert!(!should_show_for_hostname(&Some(format!(
            "!{}, !other-host",
            current_hostname
        ))));

        // Multiple negations not excluding current hostname
        assert!(should_show_for_hostname(&Some(
            "!server1, !server2".to_string()
        )));
    }
}
