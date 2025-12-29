use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserError {
    /// Required arguments or options are not provided
    MissingRequiredArguments(Vec<String>),
    MissingRequiredArgumentsForOption(String, Vec<String>),
    MissingRequiredOptions(Vec<String>),
    /// User provides an invalid option
    InvalidOptions(Vec<String>),
    /// User provides more arguments than expected
    TooManyArguments(Vec<String>),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserError::MissingRequiredArguments(args) => {
                write!(f, "error: Missing required arguments: {}", args.join(", "))
            }
            UserError::MissingRequiredArgumentsForOption(option, args) => {
                write!(
                    f,
                    "error: Missing required arguments for option {option}: {}",
                    args.join(", ")
                )
            }
            UserError::MissingRequiredOptions(opts) => {
                write!(f, "error: Missing required options: {}", opts.join(", "))
            }
            UserError::InvalidOptions(opts) => {
                write!(f, "error: Invalid options: {}", opts.join(", "))
            }
            UserError::TooManyArguments(args) => {
                write!(
                    f,
                    "error: These arguments were not expected: {}",
                    args.join(", ")
                )
            }
        }
    }
}

impl Error for UserError {}
