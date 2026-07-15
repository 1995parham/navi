use clap::Args;

use super::var;
use crate::common::shell::{self, EOF, ShellSpawnError};
use crate::prelude::*;
use std::io::{self, Read};

#[derive(Debug, Clone, Args)]
pub struct Input {}

impl Runnable for Input {
    fn run(&self) -> Result<()> {
        let mut text = String::new();
        io::stdin().read_to_string(&mut text)?;

        // `split` always yields at least one part, so only the later fields can
        // legitimately be absent. Report that instead of panicking inside the
        // fzf preview pane, where a backtrace would be all the user sees.
        let mut parts = text.split(EOF);
        let selection = parts.next().unwrap_or_default().to_owned();
        let query = parts
            .next()
            .context("Preview input is missing the query field")?
            .to_owned();
        let variable = parts
            .next()
            .context("Preview input is missing the variable field")?
            .trim()
            .to_owned();

        let input = var::Input {
            selection,
            query,
            variable,
        };

        input.run()?;

        if let Some(extra) = parts.next()
            && !extra.is_empty()
        {
            print!("");

            let mut cmd = shell::out()?;
            cmd.arg(extra);
            debug!(?cmd);
            cmd.spawn()
                .map_err(|e| ShellSpawnError::new(extra, e))?
                .wait()?;
        }

        Ok(())
    }
}
