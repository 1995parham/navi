use crate::display;
use crate::prelude::*;
use std::io::Write;
use std::process;
pub mod structures;
pub use post::process;
use skim::prelude::*;
use structures::Opts;
use structures::SuggestionType;

mod post;

pub fn call<F, R>(finder_opts: Opts, stdin_fn: F) -> Result<(String, R)>
where
    F: Fn(&mut dyn Write) -> Result<R>,
{
    // Collect input data
    let mut buffer = Vec::new();
    let return_value = stdin_fn(&mut buffer).context("Failed to collect data for skim")?;
    let input = String::from_utf8(buffer).context("Invalid utf8 data for skim")?;

    let bindings = if finder_opts.suggestion_type == SuggestionType::MultipleSelections {
        vec![
            "ctrl-j:down".to_string(),
            "ctrl-k:up".to_string(),
            "ctrl-r:toggle-all".to_string(),
        ]
    } else {
        vec!["ctrl-j:down".to_string(), "ctrl-k:up".to_string()]
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
        warn!(
            "Skim library mode: overrides may not be fully compatible: {}",
            o
        );
    }

    let options = options_builder
        .build()
        .map_err(|e| anyhow!("Failed to build skim options: {}", e))?;

    // Create item reader from input with ANSI color support enabled
    let item_reader_opts = SkimItemReaderOption::default().ansi(true);
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
