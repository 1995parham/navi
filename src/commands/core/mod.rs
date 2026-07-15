mod actor;
mod preview;
mod suggestion;

use crate::config::Source;
use crate::display;
use crate::filesystem;
use crate::finder::structures::Opts as FinderOpts;
use crate::parser::Parser;
use crate::prelude::*;
use crate::structures::fetcher::Fetcher;
use crate::welcome;

/// How many times a selection that cannot be parsed is re-prompted before giving
/// up. A selection normally fails to parse because the user picked nothing, but
/// a structural problem would otherwise retry forever.
const MAX_SELECTION_ATTEMPTS: usize = 10;

pub fn init(fetcher: Box<dyn Fetcher>) -> Result<()> {
    let config = &CONFIG;

    for attempt in 1..=MAX_SELECTION_ATTEMPTS {
        let opts = FinderOpts::snippet_default();
        debug!("opts = {opts:#?}");

        let (raw_selection, (variables, files)) = crate::finder::call(opts, |writer| {
            let mut parser = Parser::new(writer);

            let found_something = fetcher
                .fetch(&mut parser)
                .context("Failed to parse variables intended for finder")?;

            if !found_something {
                welcome::populate_cheatsheet(&mut parser)?;
            }

            Ok((Some(parser.variables), fetcher.files()))
        })
        .context("Failed getting selection and variables from finder")?;

        debug!(raw_selection = ?raw_selection);
        let extractions = display::terminal::read(&raw_selection, config.best_match());

        if let Err(e) = &extractions {
            debug!("Unable to read selection (attempt {attempt}): {e:?}");
            continue;
        }

        return actor::act(extractions, files, variables);
    }

    Err(anyhow!(
        "Failed to read a selection from the finder after {MAX_SELECTION_ATTEMPTS} attempts"
    ))
}

pub fn get_fetcher() -> Result<Box<dyn Fetcher>> {
    let source = CONFIG.source();
    debug!(source = ?source);
    match source {
        Source::Filesystem(path) => {
            let fetcher = Box::new(filesystem::Fetcher::new(path));
            Ok(fetcher)
        }
        Source::Welcome => {
            let fetcher = Box::new(welcome::Fetcher::new());
            Ok(fetcher)
        }
    }
}

pub fn main() -> Result<()> {
    let fetcher = get_fetcher()?;
    init(fetcher)
}
