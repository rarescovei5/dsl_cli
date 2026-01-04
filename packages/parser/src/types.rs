use std::{any::Any, collections::HashMap};

pub enum OptValue {
    Flag(bool),
    Args(ParsedArgs),
}
impl OptValue {
    pub fn into_flag(self) -> bool {
        match self {
            OptValue::Flag(flag) => flag,
            _ => panic!("Expected OptValue::Flag"),
        }
    }
    pub fn into_args(self) -> ParsedArgs {
        match self {
            OptValue::Args(args) => args,
            _ => panic!("Expected OptValue::Args"),
        }
    }
}
pub type ParsedArgs = HashMap<String, Option<Box<dyn Any>>>;
pub type ParsedOpts = HashMap<String, Option<OptValue>>;

// Traits
pub trait FromParsedArgs {
    fn from_parsed_args(args: ParsedArgs) -> Self;
}
pub trait FromParsedOpts {
    fn from_parsed_opts(opts: ParsedOpts) -> Self;
}
