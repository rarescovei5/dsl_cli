#![allow(unused)]

mod error;
mod parse;
mod types;

pub use parse::FromParsed;
pub use types::{Cli, CliArgument, CliCommand, CliOption, CliOptionFlags};
