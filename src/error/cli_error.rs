use crate::error::{DefinitionError, UserError};
use std::{error::Error, fmt};

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
