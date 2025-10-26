use crate::prelude::*;
pub use env::var as get;
use std::env;

// Preview-related environment variables used for internal IPC
pub const PREVIEW_INITIAL_SNIPPET: &str = "NAVI_PREVIEW_INITIAL_SNIPPET";
pub const PREVIEW_TAGS: &str = "NAVI_PREVIEW_TAGS";
pub const PREVIEW_COMMENT: &str = "NAVI_PREVIEW_COMMENT";
pub const PREVIEW_COLUMN: &str = "NAVI_PREVIEW_COLUMN";
pub const PREVIEW_DELIMITER: &str = "NAVI_PREVIEW_DELIMITER";
pub const PREVIEW_MAP: &str = "NAVI_PREVIEW_MAP";

pub fn parse<T: FromStr>(varname: &str) -> Option<T> {
    if let Ok(x) = env::var(varname) {
        x.parse::<T>().ok()
    } else {
        None
    }
}

pub fn must_get(name: &str) -> String {
    if let Ok(v) = env::var(name) {
        v
    } else {
        panic!("{name} not set")
    }
}

pub fn escape(name: &str) -> String {
    name.replace('-', "_")
}
