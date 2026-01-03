use std::collections::HashMap;

use crate::parse::parser::ParsedArgs;

pub trait TemplateArg {
    fn name(&self) -> String;
    fn optional(&self) -> bool;
    fn variadic(&self) -> bool;
}

pub fn initialize_parsed_args(template_args: &TemplateArgs) -> ParsedArgs {
    let mut parsed_args: ParsedArgs = HashMap::new();
    for arg in template_args {
        parsed_args.insert(arg.name(), None);
    }
    parsed_args
}

pub type TemplateArgs = Vec<Box<dyn TemplateArg>>;
