use crate::CliCommand;

impl CliCommand {
    pub fn args_info(&self) -> Vec<(String, String)> {
        self.arguments.iter().map(|arg| arg.info()).collect()
    }
    pub fn opts_info(&self) -> Vec<(String, String)> {
        self.options.iter().map(|opt| opt.info()).collect()
    }
    pub fn info(&self) -> (String, String) {
        let name = self.name.clone();
        let description = self.description.clone().unwrap_or_default();
        (name, description)
    }
}
