use std::{any::Any, collections::HashMap};

pub mod cli;

pub trait FromParsed {
    fn from_parsed(parsed: HashMap<String, Box<dyn Any>>) -> Self;
}
