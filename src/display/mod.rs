use crate::prelude::*;
use unicode_width::UnicodeWidthStr;

pub mod terminal;

/// Magic constants for display formatting
pub mod constants {
    /// Character used as a temporary placeholder for newlines in single-line display
    pub const NEWLINE_ESCAPE_CHAR: char = '\x15';

    /// Visual separator for multi-line commands (space + escape char + space)
    pub const LINE_SEPARATOR: &str = " \x15 ";

    /// Field separator for terminal display (invisible Braille pattern)
    pub const FIELD_SEPARATOR: &str = "  ⠀";
}

// Re-export commonly used constants for backward compatibility
const NEWLINE_ESCAPE_CHAR: char = constants::NEWLINE_ESCAPE_CHAR;
pub const LINE_SEPARATOR: &str = constants::LINE_SEPARATOR;

use std::sync::LazyLock;

pub static NEWLINE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\\s+").unwrap());
pub static VAR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\?<(\w[\w\d\-_]*)>").unwrap());

pub fn with_new_lines(txt: String) -> String {
    txt.replace(LINE_SEPARATOR, "\n")
}

pub fn fix_newlines(txt: &str) -> String {
    if txt.contains(NEWLINE_ESCAPE_CHAR) {
        (*NEWLINE_REGEX)
            .replace_all(txt.replace(LINE_SEPARATOR, "  ").as_str(), "")
            .to_string()
    } else {
        txt.to_string()
    }
}

fn limit_str(text: &str, length: usize) -> String {
    let len = UnicodeWidthStr::width(text);
    if len <= length {
        format!("{}{}", text, " ".repeat(length - len))
    } else {
        let mut new_length = length;
        let mut actual_length = 9999;
        let mut txt = text.to_owned();
        while actual_length >= length {
            txt = txt.chars().take(new_length - 1).collect::<String>();
            actual_length = UnicodeWidthStr::width(txt.as_str());
            new_length -= 1;
        }
        format!("{}…{}", txt, " ".repeat(length - actual_length - 1))
    }
}
