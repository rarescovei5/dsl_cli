use std::borrow::Cow;

use crate::{types::CliArgument, utils::is_argument};

pub struct CliOption<'a> {
    // Details
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    // Flag related
    pub short_flag: Option<Cow<'a, str>>,
    pub long_flag: Option<Cow<'a, str>>,
    pub required: bool,
    // Arg related
    pub arguments: Vec<CliArgument<'a>>,
}

impl<'a> CliOption<'a> {
    /// Create a new CliOption
    pub fn new<T>(flags_and_value: T, description: Option<T>, required: bool) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        let flags_and_value = flags_and_value.into();

        let (short_flag, long_flag) = parse_option_flags(&flags_and_value);

        let arguments = {
            let mut arguments = Vec::new();
            let mut segments = flags_and_value.split_ascii_whitespace();
            while let Some(segment) = segments.next() {
                let is_arg = is_argument(&segment.into());
                if is_arg {
                    let arg = Cow::Owned(segment.to_string());
                    arguments.push(CliArgument::new(arg, None));
                }
            }
            arguments
        };

        let name = long_flag
            .as_ref()
            .unwrap_or(short_flag.as_ref().unwrap())
            .clone();

        Self {
            // Details
            name,
            description: description.map(|d| d.into()),
            // Flag related
            short_flag,
            long_flag,
            required,
            // Argument related
            arguments,
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
