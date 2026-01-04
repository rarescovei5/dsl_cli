use std::collections::HashMap;

use args_derive::FromParsedArgs;
use opts_derive::FromParsedOpts;
use parser::{FromParsedArgs, FromParsedOpts, OptValue, ParsedArgs, ParsedOpts};

#[derive(FromParsedArgs)]
pub struct ServerConfig {
    port: u16,
}

#[cfg(test)]
mod handles_optional {
    use super::*;
    use std::any::Any;

    #[test]
    fn handles_bool() {
        #[derive(FromParsedOpts)]
        pub struct OptionalBoolOpts {
            verbose: Option<bool>,
        }
        let opts: ParsedOpts = HashMap::from([("verbose".to_string(), Some(OptValue::Flag(true)))]);
        let o = OptionalBoolOpts::from_parsed_opts(opts);
        assert_eq!(o.verbose, Some(true));
    }

    #[test]
    fn handles_args() {
        #[derive(FromParsedOpts)]
        pub struct OptionalArgsOpts {
            config: Option<ServerConfig>,
        }
        let opts: ParsedOpts = HashMap::from([(
            "config".to_string(),
            Some(OptValue::Args(HashMap::from([(
                "port".to_string(),
                Some(Box::new(8080u16) as Box<dyn Any>),
            )]))),
        )]);
        let o = OptionalArgsOpts::from_parsed_opts(opts);
        assert!(o.config.is_some());
        let config = o.config.unwrap();
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn handles_missing() {
        #[derive(FromParsedOpts)]
        pub struct OptionalOpts {
            verbose: Option<bool>,
            config: Option<ServerConfig>,
        }

        let opts: ParsedOpts =
            HashMap::from([("verbose".to_string(), None), ("config".to_string(), None)]);
        let o = OptionalOpts::from_parsed_opts(opts);
        assert_eq!(o.verbose, None);
        assert_eq!(o.config.is_none(), true);
    }
}

#[cfg(test)]
mod handles_required {
    use super::*;
    use std::any::Any;

    #[test]
    fn handles_bool() {
        #[derive(FromParsedOpts)]
        pub struct RequiredBoolOpts {
            verbose: bool,
        }
        let opts: ParsedOpts = HashMap::from([("verbose".to_string(), Some(OptValue::Flag(true)))]);
        let o = RequiredBoolOpts::from_parsed_opts(opts);
        assert_eq!(o.verbose, true);
    }

    #[test]
    fn handles_args() {
        #[derive(FromParsedOpts)]
        pub struct RequiredArgsOpts {
            config: ServerConfig,
        }
        let opts: ParsedOpts = HashMap::from([(
            "config".to_string(),
            Some(OptValue::Args(HashMap::from([(
                "port".to_string(),
                Some(Box::new(9090u16) as Box<dyn Any>),
            )]))),
        )]);
        let o = RequiredArgsOpts::from_parsed_opts(opts);
        assert_eq!(o.config.port, 9090);
    }
}
