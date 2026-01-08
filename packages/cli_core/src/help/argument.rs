use crate::{Cli, CliArgument, CliCommand, CliOption};

impl CliArgument {
    // Takes in an argument an returns something like: (<name...>,description)
    pub fn info(&self) -> (String, String) {
        let name = self.reconstruct_name();
        let description = self.description.clone().unwrap_or_default();
        (name, description)
    }
}
