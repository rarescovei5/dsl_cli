use crate::{
    parse::types::{ParsedArgs, ParsedOptions},
    types::{CliArgument, CliCommand, CliOption},
};
use std::borrow::Cow;

pub struct Cli<'a> {
    // Instance
    pub(crate) name: Option<Cow<'a, str>>,
    pub(crate) description: Option<Cow<'a, str>>,
    pub(crate) version: Option<Cow<'a, str>>,
    // Logic
    pub(crate) commands: Vec<CliCommand<'a>>,
    pub(crate) options: Vec<CliOption<'a>>,
    pub(crate) arguments: Vec<CliArgument<'a>>,
    pub(crate) action: Option<Box<dyn Fn(ParsedArgs, ParsedOptions) + 'a>>,
    // Parsed data
    pub parsed_args: ParsedArgs,
    pub parsed_options: ParsedOptions,
}

// Instance initialiaztion
impl<'a> Cli<'a> {
    /// Create a new Cli instance
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            version: None,
            commands: Vec::new(),
            options: Vec::new(),
            arguments: Vec::new(),
            action: None,
            parsed_args: ParsedArgs::new(),
            parsed_options: ParsedOptions::new(),
        }
    }
}

// Metadata methods
impl<'a> Cli<'a> {
    /// Set the name of the Cli instance
    pub fn name<T>(&mut self, name: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.name = Some(name.into());
        self
    }
    /// Set the description of the Cli instance
    pub fn description<T>(&mut self, description: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.description = Some(description.into());
        self
    }
    /// Set the version of the Cli instance
    pub fn version<T>(&mut self, version: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.version = Some(version.into());
        self
    }
}

// builder methods
impl<'a> Cli<'a> {
    /// Add a new argument to the Cli instance
    pub fn argument<T>(&mut self, name: T, description: Option<T>) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.arguments.push(CliArgument::new(name, description));
        self
    }
    /// Add a new option to the Cli instance
    pub fn option<T>(&mut self, name: T, description: Option<T>) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options.push(CliOption::new(name, description, false));
        self
    }
    /// Add a new required option to the Cli instance
    pub fn required_option<T>(&mut self, name: T, description: Option<T>) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options.push(CliOption::new(name, description, true));
        self
    }
    /// Add a new command to the Cli instance
    pub fn command<T>(&mut self, name: T) -> &mut CliCommand<'a>
    where
        T: Into<Cow<'a, str>>,
    {
        let new_command = CliCommand::new(name);
        self.commands.push(new_command);
        self.commands.last_mut().unwrap()
    }
}

// Operations
impl<'a> Cli<'a> {
    /// Set the action to be called after parsing with the parsed arguments and options
    pub fn action<F>(&mut self, action: F) -> &mut Self
    where
        F: Fn(ParsedArgs, ParsedOptions) + 'a,
    {
        self.action = Some(Box::new(action));
        self
    }
}
