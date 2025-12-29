use crate::{
    argument::CliArgument,
    option::CliOption,
    parse::types::{ParsedArgs, ParsedOptions},
};
use std::borrow::Cow;

pub struct CliCommand<'a> {
    // Command Details
    pub(crate) name: Cow<'a, str>,
    pub(crate) description: Option<Cow<'a, str>>,
    // Logic
    pub(crate) arguments: Vec<CliArgument<'a>>,
    pub(crate) options: Vec<CliOption<'a>>,
    pub(crate) action: Option<Box<dyn Fn(ParsedArgs, ParsedOptions) + 'a>>,
}

// Instance initialiaztion
impl<'a> CliCommand<'a> {
    /// Create a new CliCommand
    pub fn new<T>(name: T) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self {
            name: name.into(),
            description: None,
            arguments: Vec::new(),
            options: Vec::new(),
            action: None,
        }
    }
}

// Metadata methods
impl<'a> CliCommand<'a> {
    /// Set the description of the CliCommand
    pub fn description<T>(&mut self, description: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.description = Some(description.into());
        self
    }
}

// builder methods
impl<'a> CliCommand<'a> {
    /// Add a new argument to the CliCommand
    pub fn argument<T>(&mut self, name: T, description: Option<T>) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.arguments.push(CliArgument::new(name, description));
        self
    }
    /// Add a new option to the CliCommand
    pub fn option<T>(&mut self, name: T, description: Option<T>) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options.push(CliOption::new(name, description, false));
        self
    }
    /// Add a new required option to the CliCommand
    pub fn required_option<T>(&mut self, name: T, description: Option<T>) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options.push(CliOption::new(name, description, true));
        self
    }
}

// Operations
impl<'a> CliCommand<'a> {
    /// Set the action to be called after parsing with the parsed arguments and options
    pub fn action<F>(&mut self, action: F) -> &mut Self
    where
        F: Fn(ParsedArgs, ParsedOptions) + 'a,
    {
        self.action = Some(Box::new(action));
        self
    }
}
