use super::*;
use crate::structures::item::Item;
use crossterm::style::{Stylize, style};

pub use crate::display::constants::FIELD_SEPARATOR as DELIMITER;

pub fn write(item: &Item) -> String {
    format!(
        "{tags}{delimiter}{comment}{delimiter}{snippet}{delimiter}{tags_full}{delimiter}{comment_full}{delimiter}{snippet_full}{delimiter}{file_index}{delimiter}\n",
        tags = style(&item.tags).with(CONFIG.tag_color()),
        comment = style(&item.comment).with(CONFIG.comment_color()),
        snippet = style(&fix_newlines(&item.snippet)).with(CONFIG.snippet_color()),
        tags_full = item.tags,
        comment_full = item.comment,
        delimiter = DELIMITER,
        snippet_full = &item.snippet.trim_end_matches(LINE_SEPARATOR),
        file_index = item.file_index.unwrap_or(0),
    )
}

pub fn read(raw_snippet: &str, is_single: bool) -> Result<(&str, Item)> {
    let mut lines = raw_snippet.split('\n');
    let key = if is_single {
        "enter"
    } else {
        lines
            .next()
            .context("Key was promised but not present in `selections`")?
    };

    let mut parts = lines
        .next()
        .context("No more parts in `selections`")?
        .split(DELIMITER)
        .skip(3);

    let tags = parts.next().unwrap_or("").into();
    let comment = parts.next().unwrap_or("").into();
    let snippet = parts.next().unwrap_or("").into();
    let file_index = parts.next().unwrap_or("").parse().ok();

    let item = Item {
        tags,
        comment,
        snippet,
        file_index,
        ..Default::default()
    };

    Ok((key, item))
}
