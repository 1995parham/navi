use std::collections::HashMap;

/// Environment variables passed between processes
pub type EnvVars = HashMap<String, String>;

/// Cache of variable values during interpolation
pub type VariableCache = HashMap<String, String>;
