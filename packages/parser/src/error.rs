use std::error::Error;

#[derive(Debug)]
pub enum ParseError {
    TooManyArguments(Vec<String>),
    MissingRequiredArguments(Vec<String>),
    MissingRequiredOptions(Vec<String>),
    MissingRequiredArgumentsForOption(Vec<String>),
    InvalidOptionFlag(String),
    InvalidTypeProvided(Box<dyn Error>),
}
