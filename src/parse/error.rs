#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    TooManyArguments(Vec<String>),
    MissingRequiredArguments(Vec<String>),
    MissingRequiredOptions(Vec<String>),
    MissingRequiredArgumentsForOption(Vec<String>),
    InvalidOptionFlag(String),
}
