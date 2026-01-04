use std::{any::Any, collections::HashMap, error::Error};

use crate::ParsedArgs;

pub trait TemplateArg {
    fn name(&self) -> &str;
    fn optional(&self) -> bool;
    fn variadic(&self) -> bool;
    fn convert_one(&self, value: String) -> Result<Box<dyn Any>, Box<dyn Error>>;
    fn convert_many(&self, values: Vec<String>) -> Result<Box<dyn Any>, Box<dyn Error>>;
}

pub fn initialize_parsed_args(template_args: &TemplateArgs) -> ParsedArgs {
    let mut parsed_args: ParsedArgs = HashMap::new();
    for arg in template_args {
        parsed_args.insert(arg.name().to_string(), None);
    }
    parsed_args
}

pub type TemplateArgs = Vec<Box<dyn TemplateArg>>;
