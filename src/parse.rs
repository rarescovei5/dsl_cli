//! Parsing helpers - keeps the main parse logic clean and modular.

use crate::{
    argument::CliArgument,
    error::{CliError, UserError},
    option::CliOption,
};
use std::{collections::HashMap, iter::Peekable, vec::IntoIter};

// =============================================================================
// User Input Validation (returns errors, doesn't exit)
// =============================================================================

/// Check that all required options are present in the user's input.
pub fn validate_required_options(
    options: &[CliOption<'_>],
    user_args: &[String],
) -> Result<(), CliError> {
    let missing: Vec<String> = options
        .iter()
        .filter(|opt| opt.required)
        .filter(|opt| !option_present_in(opt, user_args))
        .map(|opt| opt.name.to_string())
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(CliError::UserError(UserError::MissingRequiredOptions(
            missing,
        )))
    }
}

/// Check that the user didn't pass any unknown options.
pub fn validate_no_unknown_options(
    options: &[CliOption<'_>],
    user_args: &[String],
) -> Result<(), CliError> {
    let invalid: Vec<String> = user_args
        .iter()
        .filter(|seg| seg.starts_with('-'))
        .filter(|seg| !is_known_option(seg, options))
        .cloned()
        .collect();

    if invalid.is_empty() {
        Ok(())
    } else {
        Err(CliError::UserError(UserError::InvalidOptions(invalid)))
    }
}

fn option_present_in(opt: &CliOption<'_>, user_args: &[String]) -> bool {
    user_args.iter().any(|seg| {
        opt.long_flag.as_ref().map(|f| f.as_ref()) == Some(seg.as_str())
            || opt.short_flag.as_ref().map(|f| f.as_ref()) == Some(seg.as_str())
    })
}

fn is_known_option(segment: &str, options: &[CliOption<'_>]) -> bool {
    options.iter().any(|opt| {
        opt.long_flag.as_ref().map(|f| f.as_ref()) == Some(segment)
            || opt.short_flag.as_ref().map(|f| f.as_ref()) == Some(segment)
    })
}

// =============================================================================
// Initialization of parsed structures
// =============================================================================

pub fn init_parsed_args(args: &[CliArgument<'_>]) -> ParsedArgs {
    let mut parsed = ParsedArgs::new();
    for arg in args {
        let value = if arg.variadic {
            ParsedArg::Multiple(None)
        } else {
            ParsedArg::Single(None)
        };
        parsed.insert(arg.name.to_string(), value);
    }
    parsed
}

pub fn init_parsed_options(options: &[CliOption<'_>]) -> ParsedOptions {
    let mut parsed = ParsedOptions::new();
    for opt in options {
        let value = if opt.arguments.is_empty() {
            ParsedOption::Boolean(false)
        } else {
            ParsedOption::Args(init_parsed_args(&opt.arguments))
        };
        parsed.insert(opt.name.to_string(), value);
    }
    parsed
}

// =============================================================================
// Token Stream - proper peekable iterator wrapper
// =============================================================================

pub struct TokenStream {
    inner: Peekable<IntoIter<String>>,
}

impl TokenStream {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            inner: args.into_iter().peekable(),
        }
    }

    pub fn peek(&mut self) -> Option<&str> {
        self.inner.peek().map(|s| s.as_str())
    }

    pub fn next(&mut self) -> Option<String> {
        self.inner.next()
    }

    pub fn peek_is_option(&mut self) -> bool {
        self.peek().map(|s| s.starts_with('-')).unwrap_or(false)
    }

    pub fn remaining_args(&mut self) -> Vec<String> {
        self.inner.clone().collect()
    }
}

// =============================================================================
// Core Parsing Logic
// =============================================================================

/// Find which option matches a given flag, if any.
pub fn find_option<'a>(segment: &str, options: &'a [CliOption<'a>]) -> Option<&'a CliOption<'a>> {
    options.iter().find(|opt| {
        opt.long_flag.as_ref().map(|f| f.as_ref()) == Some(segment)
            || opt.short_flag.as_ref().map(|f| f.as_ref()) == Some(segment)
    })
}

/// Parse arguments into a ParsedArgs map, consuming tokens from the stream.
/// Stops when hitting an option flag or running out of arguments to fill.
#[allow(dead_code)]
pub fn parse_positional_args(
    stream: &mut TokenStream,
    arg_defs: &[CliArgument<'_>],
    parsed: &mut ParsedArgs,
    start_idx: usize,
) -> Result<usize, CliError> {
    let mut idx = start_idx;

    while idx < arg_defs.len() {
        // Stop if next token is an option
        if stream.peek_is_option() || stream.peek().is_none() {
            break;
        }

        let arg_def = &arg_defs[idx];
        let token = stream.next().unwrap();

        if arg_def.variadic {
            // Variadic: consume all remaining non-option tokens
            let mut values = vec![token];
            while !stream.peek_is_option() && stream.peek().is_some() {
                values.push(stream.next().unwrap());
            }
            parsed.insert(arg_def.name.to_string(), ParsedArg::Multiple(Some(values)));
            idx += 1;
            break; // Variadic is always last (validated earlier)
        } else {
            parsed.insert(arg_def.name.to_string(), ParsedArg::Single(Some(token)));
            idx += 1;
        }
    }

    Ok(idx)
}

/// Parse option arguments (the values after a flag like `--separator <char>`).
pub fn parse_option_args(
    stream: &mut TokenStream,
    option: &CliOption<'_>,
    parsed_options: &mut ParsedOptions,
) -> Result<(), CliError> {
    if option.arguments.is_empty() {
        // Boolean flag
        parsed_options.insert(option.name.to_string(), ParsedOption::Boolean(true));
        return Ok(());
    }

    let mut opt_args = init_parsed_args(&option.arguments);
    let mut idx = 0;
    let required_count = option.arguments.iter().filter(|a| a.required).count();

    while idx < option.arguments.len() {
        if stream.peek_is_option() || stream.peek().is_none() {
            break;
        }

        let arg_def = &option.arguments[idx];
        let token = stream.next().unwrap();

        if arg_def.variadic {
            let mut values = vec![token];
            while !stream.peek_is_option() && stream.peek().is_some() {
                values.push(stream.next().unwrap());
            }
            opt_args.insert(arg_def.name.to_string(), ParsedArg::Multiple(Some(values)));
            idx += 1;
            break;
        } else {
            opt_args.insert(arg_def.name.to_string(), ParsedArg::Single(Some(token)));
            idx += 1;
        }
    }

    // Check required args were filled
    if idx < required_count {
        let missing: Vec<String> = option.arguments[idx..required_count]
            .iter()
            .filter(|a| a.required)
            .map(|a| a.name.to_string())
            .collect();
        if !missing.is_empty() {
            return Err(CliError::UserError(
                UserError::MissingRequiredArgumentsForOption(option.name.to_string(), missing),
            ));
        }
    }

    parsed_options.insert(option.name.to_string(), ParsedOption::Args(opt_args));
    Ok(())
}

// =============================================================================
// Parsed Value Types
// =============================================================================

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
