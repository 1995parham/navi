pub use crate::common::fs::ToStringExt;
pub use crate::config::CONFIG; // TODO
pub use anyhow::{Context, Error, Result, anyhow};
pub use regex::Regex;
pub use serde::de::Deserializer;
pub use serde::ser::Serializer;
pub use serde::{Deserialize, Serialize};
pub use std::any::{Any, TypeId};
pub use std::collections::{HashMap, HashSet};
pub use std::convert::{TryFrom, TryInto};
pub use std::fs::File;
pub use std::io::{BufRead, BufReader};
pub use std::path::{Path, PathBuf};
pub use std::process::Stdio;
pub use std::str::FromStr;
pub use std::sync::{Arc, Mutex, RwLock};
pub use tracing::{self, debug, error, event, info, instrument, span, subscriber, trace, warn};

pub trait Runnable {
    fn run(&self) -> Result<()>;
}
