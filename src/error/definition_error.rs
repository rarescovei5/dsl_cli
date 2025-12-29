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
