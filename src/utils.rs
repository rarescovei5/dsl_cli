use crate::types::CliArgument;
use std::borrow::Cow;

pub fn reconstruct_arg_string(arg: &CliArgument<'_>) -> String {
    let mut name = arg.name.to_string();
    if arg.variadic {
        name += "...";
    }
    if arg.required {
        name = "<".to_owned() + &name + ">";
    } else {
        name = "[".to_owned() + &name + "]";
    }
    name
}

pub fn is_argument<'a>(argument: &Cow<'a, str>) -> bool {
    argument.starts_with("<") && argument.ends_with(">")
        || argument.starts_with("[") && argument.ends_with("]")
}
