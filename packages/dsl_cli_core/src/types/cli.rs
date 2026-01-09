use super::CliCommand;
use std::path::Path;

pub struct Cli {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) description: String,
    pub(crate) commands: Vec<CliCommand>,
    // Useful for error messages
    pub(crate) executable_name: String,
    pub(crate) used_command: Option<String>,
}

impl Cli {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let name = name.into();

        let executable_name = executable_name().unwrap_or_else(|| name.clone());

        Self {
            name,
            version: version.into(),
            description: description.into(),
            commands: Vec::new(),
            executable_name,
            used_command: None,
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

fn executable_name() -> Option<String> {
    let arg0 = std::env::args().next().unwrap();

    let path = Path::new(&arg0);

    let name = path
        .file_stem()
        .or_else(|| path.file_name())
        .map(|s| s.to_string_lossy().to_string())?;

    if name.is_empty() { None } else { Some(name) }
}
