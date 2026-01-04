use std::{any::Any, error::Error, marker::PhantomData, path::PathBuf, str::FromStr};

use parser::{
    ParsedArgs, ParsedOpts, Parser, TemplateArg, TemplateArgs, TemplateOpt, TemplateOptFlags,
    TemplateOpts,
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
    let template_args: TemplateArgs = vec![
        Box::new(CliArg::<String> {
            name: "type".to_string(),
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

    assert_eq!(
        get_arg::<String>(&parsed_args, "type"),
        Some(String::from("arithmetic"))
    );
    assert_eq!(get_arg::<char>(&parsed_args, "operator"), Some('+'));
    assert_eq!(
        get_variadic_arg::<i32>(&parsed_args, "numbers"),
        Some(vec![1, 2, 3])
    );
}

#[test]
fn can_parse_opts() {
    let template_opts: TemplateOpts = vec![
        Box::new(CliOpt {
            name: "dirs".to_string(),
            optional: true,
            flags: TemplateOptFlags::Long("dirs".to_string()),
            args: vec![Box::new(CliArg::<PathBuf> {
                name: "paths".to_string(),
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
            optional: false,
            flags: TemplateOptFlags::ShortAndLong('d', "debug".to_string()),
            args: vec![],
        }),
    ];

    let env_args = env_args(&["-v", "--dirs", "src", "target", "--debug"]);

    let (_, parsed_opts) = Parser::parse_args(env_args, &vec![], &template_opts).unwrap();
    println!("{:?}", PathBuf::from("src".to_string()));

    assert_eq!(get_opt_flag(&parsed_opts, "verbose"), true);
    assert_eq!(
        get_variadic_arg::<PathBuf>(get_opt_args(&parsed_opts, "dirs"), "paths"),
        Some(vec![PathBuf::from("src"), PathBuf::from("target")])
    );
    assert_eq!(get_opt_flag(&parsed_opts, "debug"), true);
}

// ------------------------------------------------------------
// Utils
// ------------------------------------------------------------
fn env_args(args: &[&str]) -> Vec<String> {
    args.into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}
fn get_opt_flag(parsed: &ParsedOpts, key: &str) -> bool {
    parsed.get(key).unwrap().as_ref().unwrap().as_flag()
}
fn get_opt_args<'a>(parsed: &'a ParsedOpts, key: &str) -> &'a ParsedArgs {
    parsed.get(key).unwrap().as_ref().unwrap().as_args()
}
fn get_arg<T>(parsed: &ParsedArgs, key: &str) -> Option<T>
where
    T: FromStr + Clone + 'static,
{
    parsed
        .get(key)
        .unwrap()
        .as_ref()
        .and_then(|o| o.downcast_ref::<T>())
        .cloned()
}
fn get_variadic_arg<T>(parsed: &ParsedArgs, key: &str) -> Option<Vec<T>>
where
    T: FromStr + Clone + 'static,
{
    parsed
        .get(key)
        .unwrap()
        .as_ref()
        .and_then(|o| o.downcast_ref::<Vec<T>>())
        .cloned()
}
