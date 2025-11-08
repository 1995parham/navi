use crate::display;
use crate::prelude::*;
use std::io::Write;
use std::process::{self, Output};
use std::process::{Command, Stdio};
pub mod structures;
use clap::ValueEnum;
pub use post::process;
use structures::Opts;
use structures::SuggestionType;
use skim::prelude::*;

const MIN_FZF_VERSION_MAJOR: u32 = 0;
const MIN_FZF_VERSION_MINOR: u32 = 23;
const MIN_FZF_VERSION_PATCH: u32 = 1;

mod post;

#[derive(Debug, Clone, Copy, Deserialize, ValueEnum)]
pub enum FinderChoice {
    Fzf,
    Skim,
}

impl FromStr for FinderChoice {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fzf" => Ok(FinderChoice::Fzf),
            "skim" => Ok(FinderChoice::Skim),
            _ => Err("no match"),
        }
    }
}

fn parse(out: Output, opts: Opts) -> Result<String> {
    let text = match out.status.code() {
        Some(0) | Some(1) | Some(2) => {
            String::from_utf8(out.stdout).context("Invalid utf8 received from finder")?
        }
        Some(130) => process::exit(130),
        _ => {
            let err = String::from_utf8(out.stderr)
                .unwrap_or_else(|_| "<stderr contains invalid UTF-8>".to_owned());
            panic!("External command failed:\n {err}")
        }
    };

    let output = post::parse_output_single(text, opts.suggestion_type)?;
    post::process(output, opts.column, opts.delimiter.as_deref(), opts.map)
}

impl FinderChoice {
    fn check_fzf_version() -> Option<(u32, u32, u32)> {
        let output = Command::new("fzf").arg("--version").output().ok()?.stdout;
        let version_string = String::from_utf8(output).ok()?;
        let version_parts: Vec<_> = version_string.split('.').collect();
        if version_parts.len() == 3 {
            let major = version_parts[0].parse().ok()?;
            let minor = version_parts[1].parse().ok()?;
            let patch = version_parts[2].split_whitespace().next()?.parse().ok()?;
            Some((major, minor, patch))
        } else {
            None
        }
    }

    pub fn call<F, R>(&self, finder_opts: Opts, stdin_fn: F) -> Result<(String, R)>
    where
        F: Fn(&mut dyn Write) -> Result<R>,
    {
        match self {
            Self::Fzf => self.call_external(finder_opts, stdin_fn),
            Self::Skim => self.call_skim_library(finder_opts, stdin_fn),
        }
    }

    fn call_skim_library<F, R>(&self, finder_opts: Opts, stdin_fn: F) -> Result<(String, R)>
    where
        F: Fn(&mut dyn Write) -> Result<R>,
    {
        // Collect input data
        let mut buffer = Vec::new();
        let return_value = stdin_fn(&mut buffer)
            .context("Failed to collect data for skim")?;
        let input = String::from_utf8(buffer)
            .context("Invalid utf8 data for skim")?;

        let bindings = if finder_opts.suggestion_type == SuggestionType::MultipleSelections {
            vec![
                "ctrl-j:down".to_string(),
                "ctrl-k:up".to_string(),
                "ctrl-r:toggle-all".to_string(),
            ]
        } else {
            vec![
                "ctrl-j:down".to_string(),
                "ctrl-k:up".to_string(),
            ]
        };

        // Build skim options
        let mut options_builder = SkimOptionsBuilder::default();
        options_builder
            .height("100%".to_string())
            .preview(Some("".to_string()))
            .preview_window("up:3:nohidden".to_string())
            .delimiter(display::terminal::DELIMITER.to_string())
            .ansi(true)
            .bind(bindings)
            .exact(true);

        if !finder_opts.show_all_columns {
            options_builder.with_nth(vec!["1".to_string(), "2".to_string(), "3".to_string()]);
        }

        // Configure based on suggestion type
        match finder_opts.suggestion_type {
            SuggestionType::MultipleSelections => {
                options_builder.multi(true);
            }
            SuggestionType::Disabled => {
                options_builder.print_query(true);
            }
            SuggestionType::SnippetSelection => {
                options_builder.expect(vec![
                    "ctrl-y".to_string(),
                    "ctrl-o".to_string(),
                    "ctrl-e".to_string(),
                    "enter".to_string(),
                ]);
            }
            SuggestionType::SingleRecommendation => {
                options_builder
                    .print_query(true)
                    .expect(vec!["tab".to_string(), "enter".to_string()]);
            }
            _ => {}
        }

        if let Some(ref p) = finder_opts.preview {
            options_builder.preview(Some(p.clone()));
        }

        if let Some(ref q) = finder_opts.query {
            options_builder.query(Some(q.clone()));
        }

        if let Some(ref f) = finder_opts.filter {
            options_builder.filter(Some(f.clone()));
        }

        if let Some(ref d) = finder_opts.delimiter {
            options_builder.delimiter(d.clone());
        }

        if let Some(ref h) = finder_opts.header {
            options_builder.header(Some(h.clone()));
        }

        if let Some(ref p) = finder_opts.prompt {
            options_builder.prompt(p.clone());
        }

        if let Some(ref pw) = finder_opts.preview_window {
            options_builder.preview_window(pw.clone());
        }

        if finder_opts.header_lines > 0 {
            options_builder.header_lines(finder_opts.header_lines as usize);
        }

        // Apply overrides if present
        if let Some(o) = finder_opts.overrides {
            // Parse override string and apply options
            // For now, we'll log a warning that overrides are not fully supported in library mode
            warn!("Skim library mode: overrides may not be fully compatible: {}", o);
        }

        let options = options_builder.build()
            .map_err(|e| anyhow!("Failed to build skim options: {}", e))?;

        // Create item reader from input with ANSI color support enabled
        let item_reader_opts = SkimItemReaderOption::default()
            .ansi(true);
        let item_reader = SkimItemReader::new(item_reader_opts);
        let items = item_reader.of_bufread(std::io::Cursor::new(input));

        // Run skim
        let output = Skim::run_with(&options, Some(items))
            .ok_or_else(|| anyhow!("Skim was aborted or encountered an error"))?;

        // Check if user aborted
        if output.is_abort {
            // Handle abort similar to fzf exit code 130
            process::exit(130);
        }

        // Build output text based on suggestion type
        let mut result_lines = Vec::new();

        // Add query if needed
        if matches!(
            finder_opts.suggestion_type,
            SuggestionType::Disabled | SuggestionType::SingleRecommendation
        ) {
            result_lines.push(output.query);
        }

        // Add final key for snippet selection
        if finder_opts.suggestion_type == SuggestionType::SnippetSelection {
            let key_str = match output.final_key {
                Key::Ctrl('y') => "ctrl-y",
                Key::Ctrl('o') => "ctrl-o",
                Key::Ctrl('e') => "ctrl-e",
                Key::Enter => "enter",
                _ => "enter",
            };
            result_lines.push(key_str.to_string());
        } else if finder_opts.suggestion_type == SuggestionType::SingleRecommendation {
            let key_str = match output.final_key {
                Key::Tab => "tab",
                Key::Enter => "enter",
                _ => "enter",
            };
            result_lines.push(key_str.to_string());
        }

        // Add selected items
        for item in output.selected_items.iter() {
            result_lines.push(item.output().to_string());
        }

        let text = result_lines.join("\n") + "\n";

        // Parse output using existing logic
        let parsed_output = post::parse_output_single(text, finder_opts.suggestion_type)?;
        let final_output = post::process(
            parsed_output,
            finder_opts.column,
            finder_opts.delimiter.as_deref(),
            finder_opts.map,
        )?;

        Ok((final_output, return_value))
    }

    fn call_external<F, R>(&self, finder_opts: Opts, stdin_fn: F) -> Result<(String, R)>
    where
        F: Fn(&mut dyn Write) -> Result<R>,
    {
        let finder_str = match self {
            Self::Fzf => "fzf",
            Self::Skim => "sk",
        };

        if let Self::Fzf = self
            && let Some((major, minor, patch)) = Self::check_fzf_version()
            && major == MIN_FZF_VERSION_MAJOR
            && minor < MIN_FZF_VERSION_MINOR
            && patch < MIN_FZF_VERSION_PATCH
        {
            eprintln!(
                "Warning: Fzf version {major}.{minor} does not support the preview window layout used by navi.",
            );
            eprintln!(
                "Consider updating Fzf to a version >= {MIN_FZF_VERSION_MAJOR}.{MIN_FZF_VERSION_MINOR}.{MIN_FZF_VERSION_PATCH} or use a compatible layout.",
            );
            process::exit(1);
        }

        let mut command = Command::new(finder_str);
        let opts = finder_opts.clone();

        let preview_height = match self {
            FinderChoice::Skim => 3,
            _ => 2,
        };

        let bindings = if opts.suggestion_type == SuggestionType::MultipleSelections {
            ",ctrl-r:toggle-all"
        } else {
            ""
        };

        command.args([
            "--preview",
            "",
            "--preview-window",
            format!("up:{preview_height}:nohidden").as_str(),
            "--delimiter",
            display::terminal::DELIMITER.to_string().as_str(),
            "--ansi",
            "--bind",
            format!("ctrl-j:down,ctrl-k:up{bindings}").as_str(),
            "--exact",
        ]);

        if !opts.show_all_columns {
            command.args(["--with-nth", "1,2,3"]);
        }

        if !opts.prevent_select1
            && let Self::Fzf = self
        {
            command.arg("--select-1");
        }

        match opts.suggestion_type {
            SuggestionType::MultipleSelections => {
                command.arg("--multi");
            }
            SuggestionType::Disabled => {
                if let Self::Fzf = self {
                    command.args(["--print-query", "--no-select-1"]);
                };
            }
            SuggestionType::SnippetSelection => {
                command.args(["--expect", "ctrl-y,ctrl-o,ctrl-e,enter"]);
            }
            SuggestionType::SingleRecommendation => {
                command.args(["--print-query", "--expect", "tab,enter"]);
            }
            _ => {}
        }

        if let Some(p) = opts.preview {
            command.args(["--preview", &p]);
        }

        if let Some(q) = opts.query {
            command.args(["--query", &q]);
        }

        if let Some(f) = opts.filter {
            command.args(["--filter", &f]);
        }

        if let Some(d) = opts.delimiter {
            command.args(["--delimiter", &d]);
        }

        if let Some(h) = opts.header {
            command.args(["--header", &h]);
        }

        if let Some(p) = opts.prompt {
            command.args(["--prompt", &p]);
        }

        if let Some(pw) = opts.preview_window {
            command.args(["--preview-window", &pw]);
        }

        if opts.header_lines > 0 {
            command.args(["--header-lines", format!("{}", opts.header_lines).as_str()]);
        }

        if let Some(o) = opts.overrides {
            shellwords::split(&o)?
                .into_iter()
                .filter(|s| !s.is_empty())
                .for_each(|s| {
                    command.arg(s);
                });
        }

        command
            .env("SHELL", CONFIG.finder_shell())
            .envs(&opts.env_vars)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        debug!(cmd = ?command);

        let child = command.spawn();

        let mut child = match child {
            Ok(x) => x,
            Err(_) => {
                let repo = match self {
                    Self::Fzf => "https://github.com/junegunn/fzf",
                    Self::Skim => "https://github.com/lotabout/skim",
                };
                eprintln!(
                    "navi was unable to call {cmd}.
                Please make sure it's correctly installed.
                Refer to {repo} for more info.",
                    cmd = &finder_str,
                    repo = repo
                );
                process::exit(33)
            }
        };

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Unable to acquire stdin of finder"))?;

        let mut writer: Box<&mut dyn Write> = Box::new(stdin);

        let return_value = stdin_fn(&mut writer).context("Failed to pass data to finder")?;

        let out = child
            .wait_with_output()
            .context("Failed to wait for finder")?;

        let output = parse(out, finder_opts).context("Unable to get output")?;
        Ok((output, return_value))
    }
}
