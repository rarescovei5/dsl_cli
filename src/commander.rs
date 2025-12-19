use crate::command::CliCommand;
use std::borrow::Cow;

pub struct Cli<'a> {
    // Instance
    name: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
    version: Option<Cow<'a, str>>,
    // Logic
    commands: Vec<CliCommand<'a>>,
}

impl<'a> Cli<'a> {
    /// Create a new Cli instance
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            version: None,
            commands: Vec::new(),
        }
    }

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Cli Instance Initialization
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
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

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Command Logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
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
