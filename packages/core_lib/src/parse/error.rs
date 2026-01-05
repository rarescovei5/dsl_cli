#[derive(Debug)]
pub enum ParseError {
    NoCommandProvided,
    TooManyArguments(Vec<String>),
    MissingRequiredArguments(Vec<String>),
    MissingRequiredOptions(Vec<String>),
    MissingRequiredArgumentsForOption(Vec<String>),
    InvalidOptionFlag(String),
}
