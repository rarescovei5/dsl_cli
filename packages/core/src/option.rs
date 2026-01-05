use crate::argument::CliArgument;

pub enum CliOptionFlags {
    Short(char),
    Long(String),
    ShortAndLong(char, String),
}
impl PartialEq<String> for CliOptionFlags {
    fn eq(&self, other: &String) -> bool {
        match self {
            CliOptionFlags::Short(c) => c.to_string() == *other,
            CliOptionFlags::Long(s) => s == other,
            CliOptionFlags::ShortAndLong(c, s) => &c.to_string() == other || s == other,
        }
    }
}

pub struct CliOption {
    flags: CliOptionFlags,
    description: Option<String>,
    optional: bool,
    args: Vec<CliArgument>,
}

impl CliOption {
    pub fn new(flags: CliOptionFlags) -> Self {
        Self {
            flags,
            description: None,
            optional: false,
            args: Vec::new(),
        }
    }
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }
    pub fn optional(&mut self) -> &mut Self {
        self.optional = true;
        self
    }
    pub fn add_argument(&mut self, argument: CliArgument) -> &mut Self {
        self.args.push(argument);
        self
    }
}
