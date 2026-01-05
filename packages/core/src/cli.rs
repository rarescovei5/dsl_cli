use crate::command::CliCommand;

pub struct Cli {
    name: Option<String>,
    version: Option<String>,
    author: Option<String>,
    commands: Vec<CliCommand>,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            author: None,
            commands: Vec::new(),
        }
    }
    pub fn version(&mut self, version: impl Into<String>) -> &mut Self {
        self.version = Some(version.into());
        self
    }
    pub fn author(&mut self, author: impl Into<String>) -> &mut Self {
        self.author = Some(author.into());
        self
    }
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }
    pub fn command(&mut self, command_name: impl Into<String>) -> &mut CliCommand {
        let command = CliCommand::new(command_name);
        self.commands.push(command);
        self.commands.last_mut().unwrap()
    }
}
