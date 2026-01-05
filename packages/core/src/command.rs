use crate::{argument::CliArgument, option::CliOption};

pub struct CliCommand {
    name: String,
    description: Option<String>,
    options: Vec<CliOption>,
    arguments: Vec<CliArgument>,
}

impl CliCommand {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            options: Vec::new(),
            arguments: Vec::new(),
        }
    }
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }
    pub fn add_argument(&mut self, argument: CliArgument) -> &mut Self {
        self.arguments.push(argument);
        self
    }
    pub fn add_option(&mut self, option: CliOption) -> &mut Self {
        self.options.push(option);
        self
    }
}
