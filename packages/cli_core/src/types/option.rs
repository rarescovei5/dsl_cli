use cli_common::CliOptionFlags;

use super::CliArgument;

#[derive(Debug, Clone)]
pub struct CliOption {
    pub(crate) name: String,
    pub(crate) flags: CliOptionFlags,
    pub(crate) description: Option<String>,
    pub(crate) optional: bool,
    pub(crate) args: Vec<CliArgument>,
}

impl CliOption {
    pub fn new(
        name: impl Into<String>,
        flags: CliOptionFlags,
        description: Option<impl Into<String>>,
        optional: bool,
    ) -> Self {
        Self {
            name: name.into(),
            flags,
            description: description.map(|d| d.into()),
            optional,
            args: Vec::new(),
        }
    }
    pub fn add_argument(&mut self, argument: CliArgument) -> &mut Self {
        self.args.push(argument);
        self
    }
}
