use crate::CliOption;

impl CliOption {
    // Takes in an option and returns something like: (<flags> ...<name>,description)
    pub fn info(&self) -> (String, String) {
        let flags = self.flags.to_string();

        let name = self
            .args
            .iter()
            .map(|arg| arg.reconstruct_name())
            .collect::<Vec<String>>()
            .join(" ");

        let usage = [flags, name].join(" ");

        let description = self.description.clone().unwrap_or_default();

        (usage, description)
    }
}
