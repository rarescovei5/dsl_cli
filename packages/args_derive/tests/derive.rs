use std::{any::Any, collections::HashMap};

use args_derive::FromParsedArgs;
use parser::{FromParsedArgs, ParsedArgs};

#[cfg(test)]
mod handles_optional {
    use super::*;

    #[derive(FromParsedArgs)]
    struct UserArgs {
        age: Option<i32>,
    }

    #[test]
    fn can_handle_value() {
        let args: ParsedArgs =
            HashMap::from([("age".to_string(), Some(Box::new(25i32) as Box<dyn Any>))]);
        let user = UserArgs::from_parsed_args(args);
        assert_eq!(user.age, Some(25));
    }

    #[test]
    fn can_handle_missing() {
        let args: ParsedArgs = HashMap::from([("age".to_string(), None)]);
        let user = UserArgs::from_parsed_args(args);
        assert_eq!(user.age, None);
    }
}

#[cfg(test)]
mod handles_required {
    use super::*;

    #[derive(FromParsedArgs)]
    struct ServerConfig {
        ports: Vec<i32>,
    }

    #[test]
    fn can_handle_value() {
        let args: ParsedArgs = HashMap::from([(
            "ports".to_string(),
            Some(Box::new(vec![8080, 8081]) as Box<dyn Any>),
        )]);
        let server_config = ServerConfig::from_parsed_args(args);
        assert_eq!(server_config.ports, vec![8080, 8081]);
    }
}
