use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinitionError {
    /// This doesn't make sense logically because this would imply that
    /// the first argument should also be required
    /// Example: [directory] <file_name>
    RequiredArgumentAfterOptionalArgument(String, String),
    /// If we allowed this, variadic arguments would leave
    /// just enough elements for the required arguments
    /// This would be confusing for the user
    VariadicArgumentBeforeArguments(String),
}

impl fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DefinitionError::RequiredArgumentAfterOptionalArgument(optional, required) => write!(
                f,
                "error: required argument '{required}' cannot appear after optional argument '{optional}'"
            ),
            DefinitionError::VariadicArgumentBeforeArguments(variadic) => write!(
                f,
                "error: variadic argument '{variadic}' cannot appear before any other arguments"
            ),
        }
    }
}

impl Error for DefinitionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserError {
    /// Required arguments or options are not provided
    MissingRequiredArguments(Vec<String>),
    MissingRequiredOptions(Vec<String>),
    /// User provides an invalid option
    InvalidOptions(Vec<String>),
    /// User provides more arguments than expected
    TooManyArguments(usize),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserError::MissingRequiredArguments(args) => {
                write!(f, "error: Missing required arguments: {}", args.join(", "))
            }
            UserError::MissingRequiredOptions(opts) => {
                write!(f, "error: Missing required options: {}", opts.join(", "))
            }
            UserError::InvalidOptions(opts) => {
                write!(f, "error: Invalid options: {}", opts.join(", "))
            }
            UserError::TooManyArguments(count) => write!(f, "error: Too many arguments: {count}"),
        }
    }
}

impl Error for UserError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    DefinitionError(DefinitionError),
    UserError(UserError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::DefinitionError(e) => write!(f, "{e}"),
            CliError::UserError(e) => write!(f, "{e}"),
        }
    }
}

impl Error for CliError {}
