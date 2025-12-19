use crate::{argument::CliArgument, option::CliOption};
use std::{borrow::Cow, collections::HashMap};

// Both single and multiple values are represented as a list of strings
// List of parsed arguments in the order they were initialized with the command
type ParsedArgs = Vec<Vec<String>>;
// Key: long_flag (fallback: short_flag), Value: list of values
type ParsedOptions = HashMap<String, Vec<String>>;

pub struct CliCommand<'a> {
    // Command Details
    name: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    // Logic
    arguments: Vec<CliArgument<'a>>,
    options: Vec<CliOption<'a>>,
    action: Option<Box<dyn Fn(ParsedArgs, ParsedOptions) + 'a>>,
}

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

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Command Initialization
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    /// Set the description of the CliCommand
    pub fn description<T>(&mut self, description: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.description = Some(description.into());
        self
    }

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Arguments Logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    /// Add a new argument (without a description) to the CliCommand
    pub fn argument<T>(&mut self, name: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.arguments.push(CliArgument::new(name, None));
        self
    }
    /// Add a new argument (with a description) to the CliCommand
    pub fn argument_with_description<T>(&mut self, name: T, description: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.arguments
            .push(CliArgument::new(name, Some(description)));
        self
    }
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Options Logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    /// Add a new option (without a description) to the CliCommand
    pub fn option<T>(&mut self, name: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options.push(CliOption::new(name, None, false));
        self
    }
    /// Add a new option (with a description) to the CliCommand
    pub fn option_with_description<T>(&mut self, name: T, description: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options
            .push(CliOption::new(name, Some(description), false));
        self
    }
    /// Add a new required option (without a description) to the CliCommand
    pub fn required_option<T>(&mut self, name: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options.push(CliOption::new(name, None, true));
        self
    }
    /// Add a new required option (with a description) to the CliCommand
    pub fn required_option_with_description<T>(&mut self, name: T, description: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options
            .push(CliOption::new(name, Some(description), true));
        self
    }
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Action Logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    /// Add an action to be called with the parsed arguments and options
    pub fn action<F>(&mut self, action: F) -> &mut Self
    where
        F: Fn(ParsedArgs, ParsedOptions) + 'a,
    {
        self.action = Some(Box::new(action));
        self
    }
}
