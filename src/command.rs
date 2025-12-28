use crate::{
    argument::{CliArgument, ParsedArgs, ParsedOptions},
    help,
    option::CliOption,
};
use std::borrow::Cow;

pub struct CliCommand<'a> {
    // Command Details
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    // Logic
    pub arguments: Vec<CliArgument<'a>>,
    pub options: Vec<CliOption<'a>>,
    pub action: Option<Box<dyn Fn(ParsedArgs, ParsedOptions) + 'a>>,
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

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // User logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    fn help(&self) {
        let executable_name = std::env::args().next().unwrap_or_else(|| "cli".to_string());
        println!(
            "Usage: {}\n",
            help::usage_string(
                &executable_name,
                &self.arguments,
                &self.options,
                Some(&self.name)
            )
        );

        if !self.arguments.is_empty() {
            println!("Arguments: \n{}\n", help::arguments_list(&self.arguments));
        }
        if !self.options.is_empty() {
            println!("Options: \n{}\n", help::options_list(&self.options));
        }
        std::process::exit(0);
    }
}
