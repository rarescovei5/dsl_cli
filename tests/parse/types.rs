use std::any::Any;
use std::collections::HashMap;

use commander::parse::{TemplateArg, TemplateArgs, TemplateOpt, TemplateOptFlags, TemplateOpts};

#[derive(Clone)]
pub(crate) struct ArgSpec {
    pub name: String,
    pub optional: bool,
    pub variadic: bool,
}

impl TemplateArg for ArgSpec {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn optional(&self) -> bool {
        self.optional
    }
    fn variadic(&self) -> bool {
        self.variadic
    }
}

pub(crate) fn args(specs: Vec<ArgSpec>) -> TemplateArgs {
    specs
        .into_iter()
        .map(|s| Box::new(s) as Box<dyn TemplateArg>)
        .collect()
}

pub(crate) struct OptSpec {
    pub flags: TemplateOptFlags,
    pub optional: bool,
    pub args: Vec<ArgSpec>,
}

impl TemplateOpt for OptSpec {
    fn name(&self) -> String {
        match &self.flags {
            TemplateOptFlags::Short(c) => c.to_string(),
            TemplateOptFlags::Long(s) => s.clone(),
            TemplateOptFlags::ShortAndLong(_, s) => s.clone(),
        }
    }
    fn flags(&self) -> TemplateOptFlags {
        match &self.flags {
            TemplateOptFlags::Short(c) => TemplateOptFlags::Short(*c),
            TemplateOptFlags::Long(s) => TemplateOptFlags::Long(s.clone()),
            TemplateOptFlags::ShortAndLong(c, s) => TemplateOptFlags::ShortAndLong(*c, s.clone()),
        }
    }
    fn optional(&self) -> bool {
        self.optional
    }
    fn args(&self) -> TemplateArgs {
        args(self.args.clone())
    }
}

pub(crate) fn opts(specs: Vec<OptSpec>) -> TemplateOpts {
    specs
        .into_iter()
        .map(|s| Box::new(s) as Box<dyn TemplateOpt>)
        .collect()
}

pub(crate) fn get_string(map: &HashMap<String, Option<Box<dyn Any>>>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|v| v.as_ref())
        .and_then(|v| v.downcast_ref::<String>())
        .cloned()
}

pub(crate) fn get_vec_string(
    map: &HashMap<String, Option<Box<dyn Any>>>,
    key: &str,
) -> Option<Vec<String>> {
    map.get(key)
        .and_then(|v| v.as_ref())
        .and_then(|v| v.downcast_ref::<Vec<String>>())
        .cloned()
}
