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

pub struct BinOptions {
    filter: Option<String>,
    output_ending: String,
    print_query: bool,
}

pub fn call<F, R>(finder_opts: Opts, stdin_fn: F) -> Result<(String, R)>
where
    F: Fn(&mut dyn Write) -> Result<R>,
{
    // Collect input data
    let mut buffer = Vec::new();
    let return_value = stdin_fn(&mut buffer).context("Failed to collect data for skim")?;
    let input = String::from_utf8(buffer).context("Invalid utf8 data for skim")?;

    // Define key bindings based on selection mode
    const COMMON_BINDINGS: &[&str] = &["ctrl-j:down", "ctrl-k:up"];
    const MULTI_SELECT_BINDING: &str = "ctrl-r:toggle-all";

    let bindings: Vec<_> = if finder_opts.suggestion_type == SuggestionType::MultipleSelections {
        COMMON_BINDINGS
            .iter()
            .copied()
            .chain(std::iter::once(MULTI_SELECT_BINDING))
            .map(String::from)
            .collect()
    } else {
        COMMON_BINDINGS.iter().map(|&s| s.to_string()).collect()
    };

    // Build skim options
    let mut options_builder = SkimOptionsBuilder::default();
    options_builder
        .height("100%".to_string())
        .prompt("navi > ".to_string())
        .preview_window("up:3:nohidden".to_string())
        .delimiter(display::terminal::DELIMITER.to_string())
        .exact(true)
        .ansi(true)
        .bind(bindings)
        .select_1(!finder_opts.prevent_select1);

    // Configure based on suggestion type
    match finder_opts.suggestion_type {
        SuggestionType::MultipleSelections => {
            options_builder.multi(true);
        }
        SuggestionType::Disabled => {
            options_builder.print_query(true);
            options_builder.select_1(false);
        }
        SuggestionType::SnippetSelection => {
            options_builder.bind(
                [
                    "ctrl-y:accept",
                    "ctrl-o:accept",
                    "ctrl-e:accept",
                    "enter:accept",
                ]
                .map(String::from)
                .to_vec(),
            );
        }
        SuggestionType::SingleRecommendation => {
            options_builder
                .print_query(true)
                .bind(["tab:accept", "enter:accept"].map(String::from).to_vec());
        }
        _ => {}
    }

    // Apply optional finder configurations
    if let Some(preview) = &finder_opts.preview {
        options_builder.preview(Some(preview.clone()));
    }

    if let Some(query) = &finder_opts.query {
        options_builder.query(Some(query.clone()));
    }

    if let Some(filter) = &finder_opts.filter {
        options_builder.filter(Some(filter.clone()));
    }

    if let Some(delimiter) = &finder_opts.delimiter {
        options_builder.delimiter(delimiter.clone());
    }

    if let Some(header) = &finder_opts.header {
        options_builder.header(Some(header.clone()));
    }

    if let Some(prompt) = &finder_opts.prompt {
        options_builder.prompt(prompt.clone());
    }

    if let Some(preview_window) = &finder_opts.preview_window {
        options_builder.preview_window(preview_window.clone());
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

    // Create item reader with ANSI color support
    // The delimiter must be set on the item reader for column parsing to work correctly
    let delimiter = finder_opts
        .delimiter
        .as_deref()
        .unwrap_or(display::terminal::DELIMITER);

    let mut item_reader_opts = SkimItemReaderOption::default()
        .ansi(true)
        .delimiter(delimiter);

    // Control which columns to display and search (Description, Tags, Command)
    if !finder_opts.show_all_columns {
        const VISIBLE_COLUMNS: [&str; 3] = ["1", "2", "3"];
        item_reader_opts = item_reader_opts
            .with_nth(VISIBLE_COLUMNS.iter().copied()) // Display columns 1, 2, 3
            .nth(VISIBLE_COLUMNS.iter().copied()); // Search in columns 1, 2, 3
    }
    let item_reader = SkimItemReader::new(item_reader_opts);

    // Run skim
    let text = if options.filter.is_some() {
        let items = item_reader.of_bufread(std::io::Cursor::new(input));

        filter(
            &BinOptions {
                filter: options.filter.clone(),
                output_ending: String::from(if options.print0 { "\0" } else { "\n" }),
                print_query: options.print_query,
            },
            &options,
            items,
        )
    } else if input.lines().count() == 1 && options.select_1 {
        format!("{}\n", input)
    } else {
        let items = item_reader.of_bufread(std::io::Cursor::new(input));

        let output = Skim::run_with(&options, Some(items))
            .ok_or_else(|| anyhow!("Skim was aborted or encountered an error"))?;

        // Check if user aborted
        if output.is_abort {
            // Handle abort similar to skim exit code 130
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

        // Add final key for snippet selection or recommendation
        match finder_opts.suggestion_type {
            SuggestionType::SnippetSelection => {
                let key_str = match output.final_key {
                    Key::Ctrl('y') => "ctrl-y",
                    Key::Ctrl('o') => "ctrl-o",
                    Key::Ctrl('e') => "ctrl-e",
                    Key::Enter => "enter",
                    _ => "enter",
                };
                result_lines.push(key_str.to_string());
            }
            SuggestionType::SingleRecommendation => {
                let key_str = match output.final_key {
                    Key::Tab => "tab",
                    _ => "enter",
                };
                result_lines.push(key_str.to_string());
            }
            _ => {}
        }

        // Add selected items
        result_lines.extend(
            output
                .selected_items
                .iter()
                .map(|item| item.output().to_string()),
        );

        format!("{}\n", result_lines.join("\n"))
    };

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

pub fn filter(bin_option: &BinOptions, options: &SkimOptions, source: SkimItemReceiver) -> String {
    let query = bin_option.filter.clone().unwrap_or_default();

    // output query
    if bin_option.print_query {
        print!("{}{}", query, bin_option.output_ending);
    }

    let engine_factory: Box<dyn MatchEngineFactory> = if options.regex {
        Box::new(RegexEngineFactory::builder())
    } else {
        let fuzzy_engine_factory = ExactOrFuzzyEngineFactory::builder()
            .fuzzy_algorithm(options.algorithm)
            .exact_mode(options.exact)
            .build();
        Box::new(AndOrEngineFactory::new(fuzzy_engine_factory))
    };

    let engine = engine_factory.create_engine_with_case(&query, options.case);

    let mut results: Vec<(Arc<dyn skim::SkimItem>, i32)> = source
        .into_iter()
        .filter_map(|item| {
            engine
                .match_item(item.clone())
                .map(|match_result| (item, match_result.rank[0]))
        })
        .collect();

    // Sort by score in descending order (highest score first)
    results.sort_by(|a, b| b.1.cmp(&a.1));

    if let Some((item, _score)) = results.last() {
        format!("{}{}", item.output(), bin_option.output_ending)
    } else {
        String::new()
    }
}
