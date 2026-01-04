mod error;
mod parser;
mod template_arg;
mod template_opt;

pub use error::ParseError;
pub use parser::{FromParsedArgs, ParsedArgs, ParsedOpts, Parser};

pub use template_arg::{TemplateArg, TemplateArgs, initialize_parsed_args};
pub use template_opt::{TemplateOpt, TemplateOptFlags, TemplateOpts, initialize_parsed_opts};
