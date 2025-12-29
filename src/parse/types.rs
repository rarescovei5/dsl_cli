use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Str(String),
    Strs(Vec<String>),
    Bool(bool),
    Map(ParsedArgs),
}

impl Value {
    /// Chain into nested values (for options with args)
    pub fn get(&self, key: &str) -> Value {
        match self {
            Value::Map(args) => args.get(key),
            _ => Value::None,
        }
    }

    /// Extract the string value, panics if not present
    pub fn unwrap(self) -> String {
        match self {
            Value::Str(s) => s,
            Value::None => panic!("Value is None - user did not provide this argument"),
            Value::Strs(v) => panic!("Expected single value, got multiple: {:?}", v),
            Value::Bool(b) => panic!("Expected string value, got bool: {}", b),
            Value::Map(_) => panic!("Expected string value, got nested args"),
        }
    }

    /// Extract the string value or return a default
    pub fn unwrap_or(self, default: impl Into<String>) -> String {
        match self {
            Value::Str(s) => s,
            _ => default.into(),
        }
    }

    /// Extract multiple values, panics if not present
    pub fn unwrap_vec(self) -> Vec<String> {
        match self {
            Value::Strs(v) => v,
            Value::Str(s) => vec![s],
            Value::None => panic!("Value is None - user did not provide this argument"),
            Value::Bool(b) => panic!("Expected vec value, got bool: {}", b),
            Value::Map(_) => panic!("Expected vec value, got nested args"),
        }
    }

    /// Get as bool (defaults to false for None)
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => false,
        }
    }

    /// Check if value is present (user provided it)
    pub fn is_some(&self) -> bool {
        !matches!(self, Value::None)
    }

    /// Check if value is absent
    pub fn is_none(&self) -> bool {
        matches!(self, Value::None)
    }
}

// Internal storage types (not exposed to user)
#[derive(Debug, Clone)]
pub enum ParsedArg {
    Single(Option<String>),
    Multiple(Option<Vec<String>>),
}

/// Wrapper around HashMap for parsed arguments
#[derive(Debug, Clone, Default)]
pub struct ParsedArgs(pub HashMap<String, ParsedArg>);

impl ParsedArgs {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Get a value by key - returns a chainable Value
    pub fn get(&self, key: &str) -> Value {
        match self.0.get(key) {
            Some(ParsedArg::Single(Some(s))) => Value::Str(s.clone()),
            Some(ParsedArg::Single(None)) => Value::None,
            Some(ParsedArg::Multiple(Some(v))) => Value::Strs(v.clone()),
            Some(ParsedArg::Multiple(None)) => Value::None,
            None => Value::None,
        }
    }

    /// Insert a parsed argument (internal use)
    pub fn insert(&mut self, key: String, value: ParsedArg) {
        self.0.insert(key, value);
    }
}

// Internal storage for options
#[derive(Debug, Clone)]
pub enum ParsedOption {
    Boolean(bool),
    Args(ParsedArgs),
}

/// Wrapper around HashMap for parsed options
#[derive(Debug, Clone, Default)]
pub struct ParsedOptions(pub HashMap<String, ParsedOption>);

impl ParsedOptions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Get an option value - returns a chainable Value
    /// For boolean flags: returns Value::Bool
    /// For options with args: returns Value::Map (chain with .get("arg_name"))
    pub fn get(&self, key: &str) -> Value {
        match self.0.get(key) {
            Some(ParsedOption::Boolean(b)) => Value::Bool(*b),
            Some(ParsedOption::Args(args)) => Value::Map(args.clone()),
            None => Value::None,
        }
    }

    /// Insert a parsed option (internal use)
    pub fn insert(&mut self, key: String, value: ParsedOption) {
        self.0.insert(key, value);
    }
}
