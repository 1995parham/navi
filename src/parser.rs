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
    LazyLock::new(|| Regex::new(r"^\$\s*([^:]+):(.*)").unwrap());

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

/// Strips the single-character line marker (`%`, `#` or `@`) and the surrounding
/// whitespace. Operates on chars rather than bytes so that multi-byte markers'
/// neighbours are never split mid-codepoint.
fn without_prefix(line: &str) -> String {
    let mut chars = line.chars();
    chars.next();
    chars.as_str().trim().to_string()
}

#[derive(Clone, Default)]
pub struct FilterOpts {
    pub allowlist: Vec<String>,
    pub denylist: Vec<String>,
}

pub struct Parser<'a> {
    pub variables: VariableMap,
    visited_lines: HashSet<u64>,
    filter: FilterOpts,
    writer: &'a mut dyn Write,
}

/// Filters declared by `; path:`/`; os:`/`; hostname:`/`; env:` metacomments.
///
/// A metacomment always precedes the `#` comment of the item it applies to, so
/// the filters are buffered here and moved onto the item only once that item
/// starts. Assigning them on sight would attach them to the item currently being
/// accumulated, i.e. the previous one.
#[derive(Default)]
struct PendingFilters {
    path: Option<String>,
    os: Option<String>,
    hostname: Option<String>,
    env: Option<String>,
}

impl PendingFilters {
    /// Moves the buffered filters onto `item`, clearing whatever the previous
    /// item left behind so that filters never leak forwards.
    fn apply_to(&mut self, item: &mut Item) {
        item.path_filter = self.path.take();
        item.os_filter = self.os.take();
        item.hostname_filter = self.hostname.take();
        item.env_filter = self.env.take();
    }
}

fn get_current_os() -> String {
    std::env::consts::OS.to_string()
}

/// Translates a glob pattern into an anchored regex, where `**` matches across
/// path separators and `*` matches within a single component. Everything else is
/// escaped, so a pattern such as `**/.git/**` matches a literal dot rather than
/// any character.
fn glob_to_regex(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len() * 2);
    let mut literal = String::new();
    out.push('^');

    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '*' {
            literal.push(c);
            continue;
        }
        if !literal.is_empty() {
            out.push_str(&regex::escape(&literal));
            literal.clear();
        }
        if chars.peek() == Some(&'*') {
            chars.next();
            out.push_str(".*");
        } else {
            out.push_str("[^/]*");
        }
    }

    if !literal.is_empty() {
        out.push_str(&regex::escape(&literal));
    }
    out.push('$');
    out
}

fn matches_path_pattern(current_dir: &str, pattern: &str) -> bool {
    Regex::new(&glob_to_regex(pattern.trim()))
        .map(|re| re.is_match(current_dir))
        .unwrap_or(false)
}

/// Evaluates a comma-separated rule list against `matches`, where a `!` prefix
/// negates a rule.
///
/// An item is hidden as soon as a negated rule matches. Otherwise it is shown if
/// a positive rule matches, or if the list contains no positive rules at all
/// (i.e. a pure denylist such as `!windows` shows everything but Windows).
fn should_show(filter: &Option<String>, matches: impl Fn(&str) -> bool) -> bool {
    let Some(filter) = filter else {
        return true;
    };

    let mut has_positive = false;

    for rule in filter.split(',').map(str::trim).filter(|r| !r.is_empty()) {
        match rule.strip_prefix('!') {
            Some(negated) => {
                if matches(negated.trim()) {
                    return false;
                }
            }
            None => {
                has_positive = true;
                if matches(rule) {
                    return true;
                }
            }
        }
    }

    !has_positive
}

fn should_show_for_path(path_filter: &Option<String>) -> bool {
    if path_filter.is_none() {
        return true;
    }

    let Ok(current_dir) = env::current_dir() else {
        return false;
    };
    let current_dir = current_dir.to_string_lossy();

    should_show(path_filter, |pattern| {
        matches_path_pattern(&current_dir, pattern)
    })
}

fn should_show_for_os(os_filter: &Option<String>) -> bool {
    if os_filter.is_none() {
        return true;
    }

    let current_os = get_current_os();
    should_show(os_filter, |rule| rule == current_os)
}

fn get_current_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "localhost".to_string())
}

fn should_show_for_hostname(hostname_filter: &Option<String>) -> bool {
    if hostname_filter.is_none() {
        return true;
    }

    let current_hostname = get_current_hostname();
    should_show(hostname_filter, |rule| rule == current_hostname)
}

fn should_show_for_env(env_filter: &Option<String>) -> bool {
    should_show(env_filter, |rule| env::var(rule).is_ok())
}

/// Splits `--tag-rules` into an allowlist and a denylist, where a `!` prefix
/// denies a tag. Rules are trimmed, so `git, !checkout` behaves the same as
/// `git,!checkout`.
fn gen_lists(tag_rules: &str) -> FilterOpts {
    let mut allowlist = Vec::new();
    let mut denylist = Vec::new();

    for rule in tag_rules
        .split(',')
        .map(str::trim)
        .filter(|r| !r.is_empty())
    {
        match rule.strip_prefix('!') {
            Some(denied) => denylist.push(denied.trim().to_string()),
            None => allowlist.push(rule.to_string()),
        }
    }

    FilterOpts {
        allowlist,
        denylist,
    }
}

impl<'a> Parser<'a> {
    pub fn new(writer: &'a mut dyn Write) -> Self {
        let filter = match CONFIG.tag_rules() {
            Some(tr) => gen_lists(&tr),
            None => Default::default(),
        };

        Self::with_filter(writer, filter)
    }

    fn with_filter(writer: &'a mut dyn Write, filter: FilterOpts) -> Self {
        Self {
            variables: Default::default(),
            visited_lines: Default::default(),
            filter,
            writer,
        }
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

        // Filter by environment variable
        if !should_show_for_env(&item.env_filter) {
            return Ok(());
        }

        self.writer
            .write_all(display::terminal::write(item).as_bytes())
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

        let mut pending = PendingFilters::default();

        for (line_nr, line_result) in lines.enumerate() {
            let line = line_result.with_context(|| {
                format!("Failed to read line number {line_nr} in cheatsheet `{id}`")
            })?;

            if should_break {
                break;
            }

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
                pending.apply_to(&mut item);
            }
            // dependency
            else if line.starts_with('@') {
                let tags_dependency = without_prefix(&line);
                self.variables
                    .insert_dependency(&item.tags, &tags_dependency);
            }
            // path filter
            else if let Some(path) = line.strip_prefix("; path:") {
                pending.path = Some(path.trim().into());
            }
            // os filter
            else if let Some(os) = line.strip_prefix("; os:") {
                pending.os = Some(os.trim().into());
            }
            // hostname filter
            else if let Some(hostname) = line.strip_prefix("; hostname:") {
                pending.hostname = Some(hostname.trim().into());
            }
            // env filter
            else if let Some(env) = line.strip_prefix("; env:") {
                pending.env = Some(env.trim().into());
            }
            // metacomment
            else if line.starts_with(';') {
            }
            // comment
            else if line.starts_with('#') {
                should_break = self.write_cmd(&item).is_err();
                item.snippet = String::from("");
                item.comment = without_prefix(&line);
                pending.apply_to(&mut item);
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

    /// Runs `read_lines` over a cheatsheet and returns everything the parser
    /// wrote out, so that filtering can be asserted end-to-end rather than only
    /// through the `should_show_*` helpers.
    fn parse(text: &str) -> String {
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut parser = Parser::with_filter(&mut buf, FilterOpts::default());
            let lines = text.lines().map(|l| Ok(l.to_string()));
            parser
                .read_lines(lines, "test", None)
                .expect("parsing failed");
        }
        String::from_utf8(buf).expect("output is not utf8")
    }

    #[test]
    fn test_filter_applies_to_the_item_it_precedes() {
        let current_os = get_current_os();
        let text = format!(
            "% demo\n\n; os: plan9\n# alpha\necho alpha\n\n; os: {current_os}\n# bravo\necho bravo\n"
        );
        let out = parse(&text);

        assert!(
            !out.contains("alpha"),
            "a snippet for another OS must stay hidden, got: {out}"
        );
        assert!(
            out.contains("bravo"),
            "a snippet for the current OS must be shown, got: {out}"
        );
    }

    #[test]
    fn test_filters_do_not_leak_onto_later_items() {
        let text = "% demo\n\n; os: plan9\n# alpha\necho alpha\n\n# bravo\necho bravo\n";
        let out = parse(text);

        assert!(!out.contains("alpha"), "got: {out}");
        assert!(
            out.contains("bravo"),
            "an item without filters must not inherit the previous item's, got: {out}"
        );
    }

    #[test]
    fn test_filters_do_not_leak_across_tag_blocks() {
        let text = "% one\n\n; os: plan9\n# alpha\necho alpha\n\n% two\n\n# bravo\necho bravo\n";
        let out = parse(text);

        assert!(!out.contains("alpha"), "got: {out}");
        assert!(
            out.contains("bravo"),
            "a new tag block must start with no inherited filters, got: {out}"
        );
    }

    #[test]
    fn test_unfiltered_cheatsheet_is_unaffected() {
        let out = parse("% demo\n\n# alpha\necho alpha\n\n# bravo\necho bravo\n");
        assert!(out.contains("alpha"), "got: {out}");
        assert!(out.contains("bravo"), "got: {out}");
    }

    #[test]
    fn test_without_prefix() {
        assert_eq!(without_prefix("# Hello"), "Hello");
        assert_eq!(without_prefix("% git, docker"), "git, docker");
        // No space after the marker: the whole word must survive.
        assert_eq!(without_prefix("#Hello"), "Hello");
        // Multi-byte characters must not be split mid-codepoint.
        assert_eq!(without_prefix("#é unicode"), "é unicode");
        assert_eq!(without_prefix("#é"), "é");
        assert_eq!(without_prefix("#"), "");
        assert_eq!(without_prefix(""), "");
    }

    #[test]
    fn test_gen_lists_trims_whitespace() {
        // Spaces around rules must not turn a denial into an allowance.
        for rules in ["git,!checkout", "git, !checkout", " git , ! checkout "] {
            let opts = gen_lists(rules);
            assert_eq!(opts.allowlist, vec!["git".to_string()], "rules: {rules:?}");
            assert_eq!(
                opts.denylist,
                vec!["checkout".to_string()],
                "rules: {rules:?}"
            );
        }
    }

    #[test]
    fn test_glob_patterns_escape_regex_metacharacters() {
        // A literal dot must not behave as a regex wildcard.
        assert!(matches_path_pattern("/home/me/.git/hooks", "**/.git/**"));
        assert!(!matches_path_pattern("/home/me/agit/src", "**/.git/**"));
        assert!(!matches_path_pattern(
            "/home/parham/Xconfig",
            "/home/parham/.config"
        ));
        assert!(matches_path_pattern(
            "/home/parham/.config",
            "/home/parham/.config"
        ));

        // Regex metacharacters in a path must match literally rather than
        // failing to compile and silently never matching.
        assert!(matches_path_pattern("/home/u/proj(1)", "/home/u/proj(1)"));
        assert!(matches_path_pattern("/home/u/a+b", "/home/u/a+b"));
        assert!(matches_path_pattern("/home/u/a+b/src", "**/a+b/**"));
        assert!(!matches_path_pattern("/home/u/aab", "/home/u/a+b"));

        // The old implementation used a `DOUBLE_STAR` placeholder that a
        // pattern could smuggle in.
        assert!(!matches_path_pattern("/x/anything", "/x/DOUBLE_STAR"));
    }

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

    #[test]
    fn test_env_filtering() {
        // No filter - should always show
        assert!(should_show_for_env(&None));

        // Set a test env var
        // SAFETY: This test is run in a single-threaded context and the env var
        // is cleaned up at the end of the test.
        unsafe {
            env::set_var("NAVI_TEST_ENV_VAR", "test_value");
        }

        // Positive match - env var is set
        assert!(should_show_for_env(&Some("NAVI_TEST_ENV_VAR".to_string())));

        // Non-existent env var - should not show
        assert!(!should_show_for_env(&Some(
            "NAVI_NONEXISTENT_VAR".to_string()
        )));

        // Negation - exclude set env var
        assert!(!should_show_for_env(&Some(
            "!NAVI_TEST_ENV_VAR".to_string()
        )));

        // Negation - exclude non-existent env var (should show)
        assert!(should_show_for_env(&Some(
            "!NAVI_NONEXISTENT_VAR".to_string()
        )));

        // Multiple values with existing env var
        assert!(should_show_for_env(&Some(
            "NAVI_NONEXISTENT_VAR, NAVI_TEST_ENV_VAR".to_string()
        )));

        // Multiple values without any existing env var
        assert!(!should_show_for_env(&Some(
            "NAVI_NONEXISTENT_VAR1, NAVI_NONEXISTENT_VAR2".to_string()
        )));

        // Multiple negations excluding set env var
        assert!(!should_show_for_env(&Some(
            "!NAVI_TEST_ENV_VAR, !NAVI_NONEXISTENT_VAR".to_string()
        )));

        // Multiple negations not excluding any set env var
        assert!(should_show_for_env(&Some(
            "!NAVI_NONEXISTENT_VAR1, !NAVI_NONEXISTENT_VAR2".to_string()
        )));

        // Clean up
        // SAFETY: This is the cleanup for the test env var set above.
        unsafe {
            env::remove_var("NAVI_TEST_ENV_VAR");
        }
    }
}
