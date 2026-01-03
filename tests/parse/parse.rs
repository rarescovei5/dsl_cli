use crate::types::*;
use std::any::Any;
use std::collections::HashMap;

use commander::parse::{ParseError, Parser, TemplateOptFlags};

#[test]
fn parses_required_and_optional_positionals() {
    let template_args = args(vec![
        ArgSpec {
            name: "a".to_string(),
            optional: false,
            variadic: false,
        },
        ArgSpec {
            name: "b".to_string(),
            optional: true,
            variadic: false,
        },
    ]);
    let template_opts = opts(vec![]);

    let (parsed_args, _) =
        Parser::parse_args(vec!["one".into()], template_args, template_opts).unwrap();

    assert_eq!(get_string(&parsed_args, "a").as_deref(), Some("one"));
    assert_eq!(parsed_args.get("b").unwrap().is_none(), true);
}

#[test]
fn variadic_positional_stops_before_option() {
    let template_args = args(vec![ArgSpec {
        name: "files".to_string(),
        optional: false,
        variadic: true,
    }]);
    let template_opts = opts(vec![OptSpec {
        flags: TemplateOptFlags::Short('v'),
        optional: true,
        args: vec![],
    }]);

    let (parsed_args, parsed_opts) = Parser::parse_args(
        vec!["a".into(), "b".into(), "-v".into()],
        template_args,
        template_opts,
    )
    .unwrap();

    assert_eq!(
        get_vec_string(&parsed_args, "files").unwrap(),
        vec!["a".to_string(), "b".to_string()]
    );

    let verbose = parsed_opts
        .get("v")
        .and_then(|v| v.as_ref())
        .and_then(|v| v.downcast_ref::<bool>())
        .copied();
    assert_eq!(verbose, Some(true));
}

#[test]
fn parses_option_with_argument() {
    let template_args = args(vec![]);
    let template_opts = opts(vec![OptSpec {
        flags: TemplateOptFlags::Long("out".to_string()),
        optional: true,
        args: vec![ArgSpec {
            name: "path".to_string(),
            optional: false,
            variadic: false,
        }],
    }]);

    let (_parsed_args, parsed_opts) = Parser::parse_args(
        vec!["--out".into(), "file.txt".into()],
        template_args,
        template_opts,
    )
    .unwrap();

    let out_any = parsed_opts.get("out").unwrap().as_ref().unwrap();
    let out_map = out_any
        .downcast_ref::<HashMap<String, Option<Box<dyn Any>>>>()
        .unwrap();

    assert_eq!(get_string(out_map, "path").as_deref(), Some("file.txt"));
}

#[test]
fn invalid_option_flag_errors() {
    let template_args = args(vec![]);
    let template_opts = opts(vec![OptSpec {
        flags: TemplateOptFlags::Short('v'),
        optional: true,
        args: vec![],
    }]);

    let err = Parser::parse_args(vec!["--bad".into()], template_args, template_opts).unwrap_err();
    assert!(matches!(err, ParseError::InvalidOptionFlag(s) if s == "--bad"));
}

#[test]
fn missing_required_positional_errors() {
    let template_args = args(vec![ArgSpec {
        name: "a".to_string(),
        optional: false,
        variadic: false,
    }]);
    let template_opts = opts(vec![]);

    let err = Parser::parse_args(vec![], template_args, template_opts).unwrap_err();
    assert!(matches!(err, ParseError::MissingRequiredArguments(v) if v == vec!["a".to_string()]));
}

#[test]
fn missing_required_option_errors() {
    let template_args = args(vec![]);
    let template_opts = opts(vec![OptSpec {
        flags: TemplateOptFlags::Short('f'),
        optional: false,
        args: vec![],
    }]);

    let err = Parser::parse_args(vec![], template_args, template_opts).unwrap_err();
    assert!(matches!(err, ParseError::MissingRequiredOptions(v) if v == vec!["f".to_string()]));
}

#[test]
fn missing_required_option_arguments_errors() {
    let template_args = args(vec![]);
    let template_opts = opts(vec![OptSpec {
        flags: TemplateOptFlags::Long("out".to_string()),
        optional: true,
        args: vec![ArgSpec {
            name: "path".to_string(),
            optional: false,
            variadic: false,
        }],
    }]);

    let err = Parser::parse_args(vec!["--out".into()], template_args, template_opts).unwrap_err();
    assert!(
        matches!(err, ParseError::MissingRequiredArgumentsForOption(v) if v == vec!["path".to_string()])
    );
}

#[test]
fn too_many_arguments_includes_all_remaining() {
    let template_args = args(vec![ArgSpec {
        name: "a".to_string(),
        optional: false,
        variadic: false,
    }]);
    let template_opts = opts(vec![]);

    let err = Parser::parse_args(
        vec!["one".into(), "two".into(), "three".into()],
        template_args,
        template_opts,
    )
    .unwrap_err();

    assert!(
        matches!(err, ParseError::TooManyArguments(v) if v == vec!["two".to_string(), "three".to_string()])
    );
}
