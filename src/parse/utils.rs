//! Parsing helpers - keeps the main parse logic clean and modular.

use std::{iter::Peekable, vec::IntoIter};

use crate::{
    argument::CliArgument,
    error::{CliError, UserError},
    option::CliOption,
    parse::{
        cli::init_parsed_args,
        types::{ParsedArg, ParsedArgs, ParsedOption, ParsedOptions},
    },
};

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
