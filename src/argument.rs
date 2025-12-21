use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, Clone)]
pub enum ParsedArg {
    Single(Option<String>),
    Multiple(Option<Vec<String>>),
}
pub type ParsedArgs = HashMap<String, ParsedArg>;

#[derive(Debug, Clone)]
pub enum ParsedOption {
    Boolean(bool),
    Args(ParsedArgs),
}
pub type ParsedOptions = HashMap<String, ParsedOption>;

pub struct CliArgument<'a> {
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    pub multiple: bool,
    pub required: bool,
}

impl<'a> CliArgument<'a> {
    /// Create a new CliArgument
    pub fn new<T>(name: T, description: Option<T>) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        let name = name.into();

        let (name, multiple, required) = parse_argument(&name);

        Self {
            name,
            description: description.map(|d| d.into()),
            multiple,
            required,
        }
    }
}

pub fn parse_argument<'a>(argument: &Cow<'a, str>) -> (Cow<'a, str>, bool, bool) {
    if !is_argument(argument) {
        panic!("Argument name must be wrapped in:\n  -<name> for required\n  -[name] for optional");
    }

    // Check if the argument is required
    let required = argument.starts_with("<");

    // Strip name of < or [
    let mut name = argument.get(1..argument.len() - 1).unwrap();

    // Check if the argument is multiple
    let multiple = name.ends_with("...");

    // If the argument is multiple, strip the ...
    if multiple {
        name = name
            .strip_suffix("...")
            .expect("Argument name ends with '...' so there must be a '...' to strip.");
    }

    (name.to_string().into(), multiple, required)
}

pub fn is_argument<'a>(argument: &Cow<'a, str>) -> bool {
    argument.starts_with("<") && argument.ends_with(">")
        || argument.starts_with("[") && argument.ends_with("]")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn is_argument_should_work() {
        assert!(is_argument(&"<required>".into()));
        assert!(is_argument(&"[optional]".into()));
        assert!(!is_argument(&"not_an_argument".into()));
        assert!(!is_argument(&"<missing_end".into()));
        assert!(!is_argument(&"missing_start>".into()));
    }

    #[test]
    fn parse_argument_required_should_work() {
        let arg = Cow::from("<name>");
        let (name, multiple, required) = parse_argument(&arg);
        assert_eq!(name, "name");
        assert!(!multiple);
        assert!(required);
    }

    #[test]
    fn parse_argument_optional_should_work() {
        let arg = Cow::from("[name]");
        let (name, multiple, required) = parse_argument(&arg);
        assert_eq!(name, "name");
        assert!(!multiple);
        assert!(!required);
    }

    #[test]
    fn parse_argument_multiple_required_should_work() {
        let arg = Cow::from("<name...>");
        let (name, multiple, required) = parse_argument(&arg);
        assert_eq!(name, "name");
        assert!(multiple);
        assert!(required);
    }

    #[test]
    fn parse_argument_multiple_optional_should_work() {
        let arg = Cow::from("[name...]");
        let (name, multiple, required) = parse_argument(&arg);
        assert_eq!(name, "name");
        assert!(multiple);
        assert!(!required);
    }

    #[test]
    #[should_panic(expected = "Argument name must be wrapped in")]
    fn parse_argument_invalid_should_panic() {
        let arg = Cow::from("invalid");
        parse_argument(&arg);
    }
}
