mod error;
mod parser;
mod template_arg;
mod template_opt;

pub use parser::Parser;

pub use error::ParseError;
pub use template_arg::{TemplateArg, TemplateArgs};
pub use template_opt::{TemplateOpt, TemplateOptFlags, TemplateOpts};

use template_arg::initialize_parsed_args;
use template_opt::initialize_parsed_opts;
