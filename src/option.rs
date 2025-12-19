use std::borrow::Cow;

use crate::argument::{is_argument, parse_argument};

pub struct CliOption<'a> {
    // Details
    description: Option<Cow<'a, str>>,
    // Flag related
    short_flag: Option<Cow<'a, str>>,
    long_flag: Option<Cow<'a, str>>,
    required: bool,
    // Arg related
    arg_name: Option<Cow<'a, str>>,
    required_arg: bool,
    multiple: bool,
}

impl<'a> CliOption<'a> {
    /// Create a new CliOption
    pub fn new<T>(flags_and_value: T, description: Option<T>, required: bool) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        let flags_and_value = flags_and_value.into();

        let (short_flag, long_flag) = parse_option_flags(&flags_and_value);

        let (arg_name, multiple, required_arg) = {
            let last_segment = flags_and_value
                .split_ascii_whitespace()
                .last()
                .expect("There will always be a last element since we split by whitespace.");
            let is_arg = is_argument(&last_segment.into());

            if is_arg {
                let arg = Cow::Owned(last_segment.to_string());
                let (arg_name, multiple, required) = parse_argument(&arg);
                (Some(arg_name), multiple, required)
            } else {
                (None, false, required)
            }
        };

        Self {
            // Details
            description: description.map(|d| d.into()),
            // Flag related
            short_flag,
            long_flag,
            required,
            // Argument related
            arg_name,
            multiple,
            required_arg,
        }
    }
}

fn parse_option_flags<'a>(
    flags_and_value: &Cow<'a, str>,
) -> (Option<Cow<'a, str>>, Option<Cow<'a, str>>) {
    let mut short_flag = None;
    let mut long_flag = None;

    for segment in flags_and_value.split_ascii_whitespace() {
        if segment.starts_with("--") {
            long_flag = Some(segment.to_string().into());
        } else if segment.starts_with("-") {
            let short_flag_len = segment.len();
            if !(short_flag_len >= 2) {
                panic!(
                    "Short flag must be of form -X, where X is a single character. Found: {segment}"
                );
            }
            short_flag = Some(
                segment
                    .get(0..2)
                    .expect("Short flag must be of form -X.")
                    .to_string()
                    .into(),
            );
        } else {
            break;
        }
    }

    (short_flag, long_flag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_option_flags_both_should_work() {
        let flags = Cow::from("-s, --long");
        let (short, long) = parse_option_flags(&flags);
        assert_eq!(short, Some("-s".into()));
        assert_eq!(long, Some("--long".into()));
    }

    #[test]
    fn parse_option_flags_only_short_should_work() {
        // Note: implementation expects a comma even if it's the only flag
        let flags = Cow::from("-s");
        let (short, long) = parse_option_flags(&flags);
        assert_eq!(short, Some("-s".into()));
        assert_eq!(long, None);
    }

    #[test]
    fn parse_option_flags_only_long_should_work() {
        let flags = Cow::from("--long");
        let (short, long) = parse_option_flags(&flags);
        assert_eq!(short, None);
        assert_eq!(long, Some("--long".into()));
    }

    #[test]
    fn parse_option_flags_with_value_should_work() {
        let flags = Cow::from("-s, --long <value>");
        let (short, long) = parse_option_flags(&flags);
        assert_eq!(short, Some("-s".into()));
        assert_eq!(long, Some("--long".into()));
    }
}
