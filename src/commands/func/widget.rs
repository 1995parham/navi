use crate::prelude::*;
use std::io::{self, Read};

pub fn last_command() -> Result<()> {
    let mut text = String::new();
    io::stdin().read_to_string(&mut text)?;

    const REPLACEMENTS: [(&str, &str); 3] = [("||", "ග"), ("|", "ඛ"), ("&&", "ඝ")];

    let parts = shellwords::split(&text)
        .unwrap_or_else(|_| text.split('|').map(|s| s.to_string()).collect());

    for p in parts {
        for (pattern, escaped) in REPLACEMENTS {
            if p.contains(pattern) && p != pattern && p != format!("{pattern}{pattern}") {
                let replacement = p.replace(pattern, escaped);
                text = text.replace(&p, &replacement);
            }
        }
    }

    let mut extracted = text.clone();

    for (pattern, _) in REPLACEMENTS {
        let mut new_parts = text.rsplit(pattern);
        if let Some(extracted_attempt) = new_parts.next()
            && extracted_attempt.len() <= extracted.len()
        {
            extracted = extracted_attempt.to_string();
        }
    }

    for (pattern, escaped) in REPLACEMENTS {
        text = text.replace(escaped, pattern);
        extracted = extracted.replace(escaped, pattern);
    }

    println!("{}", extracted.trim_start());

    Ok(())
}
