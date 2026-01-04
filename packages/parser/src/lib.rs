mod error;
mod parser;
mod template_arg;
mod template_opt;
mod types;

pub use error::ParseError;
pub use parser::Parser;
pub use types::{FromParsedArgs, FromParsedOpts, OptValue, ParsedArgs, ParsedOpts};

pub use template_arg::{TemplateArg, TemplateArgs, initialize_parsed_args};
pub use template_opt::{TemplateOpt, TemplateOptFlags, TemplateOpts, initialize_parsed_opts};
