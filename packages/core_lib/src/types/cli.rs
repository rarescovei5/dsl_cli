use super::CliCommand;

pub struct Cli {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) author: String,
    pub(crate) commands: Vec<CliCommand>,
}

impl Cli {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            author: author.into(),
            commands: Vec::new(),
        }
    }
    pub fn add_command(
        &mut self,
        command_name: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> &mut CliCommand {
        let command = CliCommand::new(command_name, description);
        self.commands.push(command);
        self.commands.last_mut().unwrap()
    }
}
