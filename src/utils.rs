use crate::argument::CliArgument;

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
