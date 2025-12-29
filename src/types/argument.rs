use std::borrow::Cow;

#[derive(Clone)]
pub struct CliArgument<'a> {
    pub(crate) name: Cow<'a, str>,
    pub(crate) description: Option<Cow<'a, str>>,
    pub(crate) variadic: bool,
    pub(crate) required: bool,
}

// Instance initialiaztion
impl<'a> CliArgument<'a> {
    /// Create a new CliArgument
    pub fn new<T>(argument: T, description: Option<T>) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        let argument = argument.into();

        let (name, variadic, required) = parse_argument(&argument);

        Self {
            name,
            description: description.map(|d| d.into()),
            variadic,
            required,
        }
    }
}

fn parse_argument<'a>(argument: &Cow<'a, str>) -> (Cow<'a, str>, bool, bool) {
    if !is_argument(argument) {
        panic!("Argument name must be wrapped in:\n  -<name> for required\n  -[name] for optional");
    }

    // Check if the argument is required
    let required = argument.starts_with("<");

    // Strip name of < or [
    let mut name = argument.get(1..argument.len() - 1).unwrap();

    // Check if the argument is variadic
    let variadic = name.ends_with("...");

    // If the argument is variadic, strip the ...
    if variadic {
        name = name
            .strip_suffix("...")
            .expect("Argument name ends with '...' so there must be a '...' to strip.");
    }

    (name.to_string().into(), variadic, required)
}

pub fn is_argument<'a>(argument: &Cow<'a, str>) -> bool {
    argument.starts_with("<") && argument.ends_with(">")
        || argument.starts_with("[") && argument.ends_with("]")
}
