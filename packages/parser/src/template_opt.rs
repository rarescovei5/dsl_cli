use std::collections::HashMap;

use crate::{OptValue, ParsedOpts, TemplateArgs, initialize_parsed_args};

#[derive(Clone, Debug)]
pub enum TemplateOptFlags {
    Short(char),
    Long(String),
    ShortAndLong(char, String),
}

impl PartialEq<String> for TemplateOptFlags {
    fn eq(&self, other: &String) -> bool {
        match self {
            TemplateOptFlags::Short(short_flag) => {
                ("-".to_string() + &short_flag.to_string()) == *other
            }
            TemplateOptFlags::Long(long_flag) => ("--".to_string() + long_flag) == *other,
            TemplateOptFlags::ShortAndLong(short_flag, long_flag) => {
                ("-".to_string() + &short_flag.to_string()) == *other
                    || ("--".to_string() + long_flag) == *other
            }
        }
    }
}

pub trait TemplateOpt {
    fn name(&self) -> &str;
    fn flags(&self) -> &TemplateOptFlags;
    fn optional(&self) -> bool;
    fn args(&self) -> &TemplateArgs;
}

pub fn initialize_parsed_opts(template_opts: &TemplateOpts) -> ParsedOpts {
    let mut parsed_opts: ParsedOpts = HashMap::new();
    for opt in template_opts {
        if opt.args().is_empty() {
            parsed_opts.insert(opt.name().to_string(), None);
        } else {
            parsed_opts.insert(
                opt.name().to_string(),
                Some(OptValue::Args(initialize_parsed_args(&opt.args()))),
            );
        }
    }
    parsed_opts
}

pub type TemplateOpts = Vec<Box<dyn TemplateOpt>>;
