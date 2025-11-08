mod cli;
mod toml;

use crate::commands::func::Func;
use crate::prelude::debug;
pub use cli::*;
use crossterm::style::Color;
use toml::TomlConfig;

use std::sync::LazyLock;

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::new);
#[derive(Debug)]
pub struct Config {
    toml: TomlConfig,
    clap: ClapConfig,
}

impl Config {
    pub fn new() -> Self {
        let toml = TomlConfig::get().unwrap_or_else(|e| {
            eprintln!("Error parsing config file: {e}");
            eprintln!("Fallbacking to default one...");
            eprintln!();
            TomlConfig::default()
        });
        let clap = ClapConfig::new();
        Self { toml, clap }
    }

    pub fn best_match(&self) -> bool {
        self.clap.best_match
    }

    pub fn prevent_interpolation(&self) -> bool {
        self.clap.prevent_interpolation
    }

    pub fn cmd(&self) -> Option<&Command> {
        self.clap.cmd.as_ref()
    }

    pub fn source(&self) -> Source {
        if let Some(Command::Fn(input)) = self.cmd() {
            if let Func::Welcome = input.func {
                Source::Welcome
            } else {
                Source::Filesystem(self.path())
            }
        } else {
            Source::Filesystem(self.path())
        }
    }

    pub fn path(&self) -> Option<String> {
        if let Some(ref path) = self.clap.path {
            debug!("CLAP PATH: {}", path);
        }

        self.clap
            .path
            .clone()
            .or_else(|| {
                let p = self.toml.cheats.paths.clone();

                if p.is_empty() {
                    None
                } else {
                    debug!("MULTIPLE TOML PATH: {}", p.as_slice().join(","));
                    Some(p.join(crate::filesystem::JOIN_SEPARATOR))
                }
            })
            .or_else(|| {
                if let Some(ref path) = self.toml.cheats.path {
                    debug!("DEPRECATED UNIQUE TOML PATH: {}", path);
                }

                self.toml.cheats.path.clone()
            })
            .or_else(|| {
                debug!("No specific path given!");

                None
            })
    }

    pub fn fzf_overrides(&self) -> Option<String> {
        self.clap
            .fzf_overrides
            .clone()
            .or_else(|| self.toml.finder.overrides.clone())
    }

    pub fn fzf_overrides_var(&self) -> Option<String> {
        self.clap
            .fzf_overrides_var
            .clone()
            .or_else(|| self.toml.finder.overrides_var.clone())
    }

    pub fn delimiter_var(&self) -> Option<String> {
        self.toml.finder.delimiter_var.clone()
    }

    pub fn shell(&self) -> String {
        self.toml.shell.command.clone()
    }

    pub fn finder_shell(&self) -> String {
        self.toml
            .shell
            .finder_command
            .clone()
            .unwrap_or_else(|| self.toml.shell.command.clone())
    }

    pub fn tag_rules(&self) -> Option<String> {
        self.clap
            .tag_rules
            .clone()
            .or_else(|| self.toml.search.tags.clone())
    }

    pub fn tag_color(&self) -> Color {
        self.toml.style.tag.color.get()
    }

    pub fn comment_color(&self) -> Color {
        self.toml.style.comment.color.get()
    }

    pub fn snippet_color(&self) -> Color {
        self.toml.style.snippet.color.get()
    }

    #[cfg(feature = "disable-command-execution")]
    fn print(&self) -> bool {
        true
    }

    #[cfg(not(feature = "disable-command-execution"))]
    fn print(&self) -> bool {
        self.clap.print
    }

    pub fn action(&self) -> Action {
        if self.print() {
            Action::Print
        } else {
            Action::Execute
        }
    }

    pub fn get_query(&self) -> Option<String> {
        let q = self.clap.query.clone();
        if q.is_some() {
            return q;
        }
        if self.best_match() {
            Some(String::from(""))
        } else {
            None
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
