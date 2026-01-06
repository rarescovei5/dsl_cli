#[derive(Debug)]
pub enum ParseError {
    InvalidCommand,
    TooManyArguments(Vec<String>),
    MissingRequiredArguments(Vec<String>),
    MissingRequiredOptions(Vec<String>),
    MissingRequiredArgumentsForOption(Vec<String>),
    InvalidOptionFlag(String),
}
