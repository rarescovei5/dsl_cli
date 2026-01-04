use std::{any::Any, error::Error, marker::PhantomData, path::PathBuf, str::FromStr};

use args_derive::FromParsedArgs;
use opts_derive::FromParsedOpts;
use parser::{
    FromParsedArgs, FromParsedOpts, ParsedArgs, ParsedOpts, Parser, TemplateArg, TemplateArgs,
    TemplateOpt, TemplateOptFlags, TemplateOpts,
};

pub struct CliArg<T>
where
    T: FromStr,
{
    name: String,
    optional: bool,
    variadic: bool,
    ty: PhantomData<T>,
}

impl<T> TemplateArg for CliArg<T>
where
    T: FromStr + 'static,
    T::Err: std::error::Error + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }
    fn optional(&self) -> bool {
        self.optional
    }
    fn variadic(&self) -> bool {
        self.variadic
    }
    fn convert_one(&self, value: String) -> Result<Box<dyn Any>, Box<dyn Error>> {
        let parsed: T = value.parse::<T>().map_err(|e| Box::new(e))?;
        Ok(Box::new(parsed))
    }
    fn convert_many(&self, values: Vec<String>) -> Result<Box<dyn Any>, Box<dyn Error>> {
        let mut out = Vec::<T>::with_capacity(values.len());
        for v in values {
            let parsed: T = v.parse::<T>().map_err(|e| Box::new(e))?;
            out.push(parsed);
        }
        Ok(Box::new(out))
    }
}

pub struct CliOpt {
    name: String,
    flags: TemplateOptFlags,
    optional: bool,
    args: TemplateArgs,
}

impl TemplateOpt for CliOpt {
    fn args(&self) -> &TemplateArgs {
        &self.args
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn flags(&self) -> &TemplateOptFlags {
        &self.flags
    }
    fn optional(&self) -> bool {
        self.optional
    }
}

#[test]
fn can_parse_args() {
    #[derive(FromParsedArgs)]
    struct Args {
        ty: String,
        operator: char,
        numbers: Vec<i32>,
    }

    let template_args: TemplateArgs = vec![
        Box::new(CliArg::<String> {
            name: "ty".to_string(),
            optional: false,
            variadic: false,
            ty: PhantomData,
        }),
        Box::new(CliArg::<char> {
            name: "operator".to_string(),
            optional: false,
            variadic: false,
            ty: PhantomData,
        }),
        Box::new(CliArg::<i32> {
            name: "numbers".to_string(),
            optional: true,
            variadic: true,
            ty: PhantomData,
        }),
    ];

    let env_args = env_args(&["arithmetic", "+", "1", "2", "3"]);
    let (parsed_args, _) = Parser::parse_args(env_args, &template_args, &vec![]).unwrap();

    let parsed_args = Args::from_parsed_args(parsed_args);

    assert_eq!(parsed_args.ty, String::from("arithmetic"));
    assert_eq!(parsed_args.operator, '+');
    assert_eq!(parsed_args.numbers, vec![1, 2, 3]);
}

#[test]
fn can_parse_opts() {
    #[derive(FromParsedArgs)]
    struct DirsOpt {
        strings: Vec<PathBuf>,
    }

    #[derive(FromParsedOpts)]
    struct Opts {
        dirs: DirsOpt,
        verbose: bool,
        debug: Option<bool>,
    }

    let template_opts: TemplateOpts = vec![
        Box::new(CliOpt {
            name: "dirs".to_string(),
            optional: true,
            flags: TemplateOptFlags::Long("dirs".to_string()),
            args: vec![Box::new(CliArg::<PathBuf> {
                name: "strings".to_string(),
                optional: false,
                variadic: true,
                ty: PhantomData,
            })],
        }),
        Box::new(CliOpt {
            name: "verbose".to_string(),
            optional: false,
            flags: TemplateOptFlags::Short('v'),
            args: vec![],
        }),
        Box::new(CliOpt {
            name: "debug".to_string(),
            optional: true,
            flags: TemplateOptFlags::ShortAndLong('d', "debug".to_string()),
            args: vec![],
        }),
    ];

    let env_args = env_args(&["-v", "--dirs", "src", "target", "--debug"]);

    let (_, parsed_opts) = Parser::parse_args(env_args, &vec![], &template_opts).unwrap();
    println!("{:?}", PathBuf::from("src".to_string()));

    let parsed_opts = Opts::from_parsed_opts(parsed_opts);

    assert_eq!(parsed_opts.verbose, true);
    assert_eq!(parsed_opts.debug, Some(true));
    assert_eq!(
        parsed_opts.dirs.strings,
        vec![PathBuf::from("src"), PathBuf::from("target")]
    );
}

#[test]
fn can_parse_args_and_opts() {
    #[derive(FromParsedArgs)]
    struct Args {
        operator: String,
        numbers: Option<Vec<i32>>,
    }

    let template_args: TemplateArgs = vec![
        Box::new(CliArg::<String> {
            name: "operator".to_string(),
            optional: false,
            variadic: false,
            ty: PhantomData,
        }),
        Box::new(CliArg::<i32> {
            name: "numbers".to_string(),
            optional: true,
            variadic: true,
            ty: PhantomData,
        }),
    ];

    #[derive(FromParsedArgs)]
    struct ConfigOpt {
        format: String,
        timeout: i32,
    }

    #[derive(FromParsedOpts)]
    struct Opts {
        verbose: Option<bool>,
        dry_run: Option<bool>,
        config: ConfigOpt,
    }

    let template_opts: TemplateOpts = vec![
        Box::new(CliOpt {
            name: "verbose".to_string(),
            optional: true,
            flags: TemplateOptFlags::Short('v'),
            args: vec![],
        }),
        Box::new(CliOpt {
            name: "dry_run".to_string(),
            optional: true,
            flags: TemplateOptFlags::Long("dry-run".to_string()),
            args: vec![],
        }),
        Box::new(CliOpt {
            name: "config".to_string(),
            optional: false,
            flags: TemplateOptFlags::ShortAndLong('c', "config".to_string()),
            args: vec![
                Box::new(CliArg::<String> {
                    name: "format".to_string(),
                    optional: false,
                    variadic: false,
                    ty: PhantomData,
                }),
                Box::new(CliArg::<i32> {
                    name: "timeout".to_string(),
                    optional: false,
                    variadic: false,
                    ty: PhantomData,
                }),
            ],
        }),
    ];

    let env_args = env_args(&["sum", "1", "2", "3", "-v", "--config", "json", "30"]);

    let (parsed_args_map, parsed_opts_map) =
        Parser::parse_args(env_args, &template_args, &template_opts).unwrap();

    let args = Args::from_parsed_args(parsed_args_map);
    let opts = Opts::from_parsed_opts(parsed_opts_map);

    assert_eq!(args.operator, "sum");
    assert_eq!(args.numbers, Some(vec![1, 2, 3]));

    assert_eq!(opts.verbose, Some(true));
    assert_eq!(opts.dry_run, None); // Optional flag not present
    assert_eq!(opts.config.format, "json");
    assert_eq!(opts.config.timeout, 30);
}

fn env_args(args: &[&str]) -> Vec<String> {
    args.into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}
