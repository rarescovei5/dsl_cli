use std::{any::Any, collections::HashMap, iter::Peekable};

use super::error::ParseError;
use crate::{Cli, CliArgument, CliOption, FromParsed};

// The Box<dyn Any> represents either None or a String
type ParsedArgs = HashMap<String, Box<dyn Any>>;
type ParsedOpts = HashMap<String, Box<dyn Any>>;

impl Cli {
    pub fn parse(
        &self,
        command_name: String,
        env_args: Vec<String>,
    ) -> Result<(ParsedArgs, ParsedOpts), ParseError> {
        let command_name = if command_name.is_empty() {
            "cli".to_owned()
        } else {
            command_name
        };

        let command_def = match self.commands.iter().find(|cmd| cmd.name == command_name) {
            Some(cmd) => cmd,
            None => return Err(ParseError::InvalidCommand),
        };

        let (parsed_args, parsed_opts) = Self::parse_args(
            env_args,
            command_def.arguments.clone(),
            command_def.options.clone(),
        )?;

        Ok((parsed_args, parsed_opts))
    }

    fn parse_args(
        env_args: Vec<String>,
        template_args: Vec<CliArgument>,
        template_opts: Vec<CliOption>,
    ) -> Result<(ParsedArgs, ParsedOpts), ParseError> {
        let mut parsed_args = Self::initialize_parsed_args(&template_args);
        let mut parsed_opts = Self::initialize_parsed_opts(&template_opts);
        let mut tokens = env_args.into_iter().peekable();
        let mut positional_idx = 0;

        while let Some(token) = tokens.next() {
            if Self::is_option_token(&token) {
                // Check if the option is included in the template
                if !template_opts.iter().any(|opt| opt.flags == token) {
                    return Err(ParseError::InvalidOptionFlag(token));
                }

                let opt_def = template_opts.iter().find(|opt| opt.flags == token).unwrap();

                // Option has no arguments = flag-only option
                if opt_def.args.is_empty() {
                    parsed_opts.insert(opt_def.name.clone(), Box::new("true".to_string()));
                    continue;
                }

                // Handle positional arguments for option
                let opt_args = opt_def.args.clone();
                let mut parsed_opt_args = Self::initialize_parsed_args(&opt_args);
                let mut idx = 0;

                while idx < parsed_opt_args.len() {
                    if tokens.peek().is_none() || Self::is_option_token(tokens.peek().unwrap()) {
                        break;
                    }

                    let arg_def = &opt_args[idx];
                    let token = tokens.next().unwrap();
                    let parsed_value = Self::parse_arg(arg_def, token, &mut tokens)?;

                    // If the option only has one argument, insert the value into the option directly
                    if parsed_opt_args.len() == 1 {
                        parsed_opts.insert(opt_def.name.clone(), parsed_value);
                    } else {
                        parsed_opt_args.insert(arg_def.name.clone(), parsed_value);
                    }

                    idx += 1;
                }

                Self::check_for_missing_required_args(&opt_args, idx, true)?;

                if opt_def.args.len() > 1 {
                    parsed_opts.insert(opt_def.name.clone(), Box::new(parsed_opt_args));
                }
            } else {
                // Check if we've gone past the number of positional arguments
                if positional_idx >= template_args.len() {
                    let mut remaining_args = vec![token];
                    remaining_args.extend(tokens);
                    return Err(ParseError::TooManyArguments(remaining_args));
                }

                // Handle positional arguments
                let arg_def = &template_args[positional_idx];
                let parsed_value = Self::parse_arg(arg_def, token, &mut tokens)?;
                parsed_args.insert(arg_def.name.clone(), parsed_value);
                positional_idx += 1;
            }
        }

        Self::check_for_missing_required_args(&template_args, positional_idx, false)?;
        Self::check_for_missing_required_opts(&parsed_opts, &template_opts)?;

        Ok((parsed_args, parsed_opts))
    }

    // ------------------------------------------------------------
    // Utils
    // ------------------------------------------------------------
    fn parse_arg(
        arg_def: &CliArgument,
        current_token: String,
        tokens: &mut Peekable<std::vec::IntoIter<String>>,
    ) -> Result<Box<dyn Any>, ParseError> {
        if arg_def.variadic {
            let mut values = vec![current_token];
            while tokens.peek().is_some() && !Self::is_option_token(tokens.peek().unwrap()) {
                values.push(tokens.next().unwrap());
            }

            Ok(Box::new(values))
        } else {
            Ok(Box::new(current_token))
        }
    }

    // ------------------------------------------------------------
    // Validation methods
    // ------------------------------------------------------------
    fn check_for_missing_required_opts(
        parsed_opts: &ParsedOpts,
        template_opts: &Vec<CliOption>,
    ) -> Result<(), ParseError> {
        let required_opts = template_opts.iter().filter(|opt| !opt.optional);

        let mut missing_required_opts = Vec::new();

        for opt in required_opts {
            match opt.args.len() {
                0 => {
                    if parsed_opts
                        .get(&opt.name.clone())
                        .unwrap()
                        .downcast_ref::<bool>()
                        .is_none()
                    {
                        missing_required_opts.push(opt.name.clone());
                    }
                }
                1 => {
                    if parsed_opts
                        .get(&opt.name.clone())
                        .unwrap()
                        .downcast_ref::<String>()
                        .is_none()
                    {
                        missing_required_opts.push(opt.name.clone());
                    }
                }
                _ => {
                    if parsed_opts
                        .get(&opt.name.clone())
                        .unwrap()
                        .downcast_ref::<HashMap<String, Box<dyn Any>>>()
                        .is_none()
                    {
                        missing_required_opts.push(opt.name.clone());
                    }
                }
            };
        }

        if !missing_required_opts.is_empty() {
            return Err(ParseError::MissingRequiredOptions(missing_required_opts));
        }

        Ok(())
    }
    fn check_for_missing_required_args(
        template_args: &Vec<CliArgument>,
        positional_idx: usize,
        are_opt_args: bool,
    ) -> Result<(), ParseError> {
        if positional_idx < template_args.iter().filter(|arg| !arg.optional).count() {
            let last_required_arg_idx = template_args.len()
                - template_args
                    .iter()
                    .rev()
                    .position(|arg| !arg.optional)
                    .unwrap()
                - 1;
            let missing_args = template_args[positional_idx..last_required_arg_idx]
                .iter()
                .filter(|arg| !arg.optional)
                .map(|arg| arg.name.clone())
                .collect::<Vec<String>>();
            if are_opt_args {
                return Err(ParseError::MissingRequiredArgumentsForOption(missing_args));
            } else {
                return Err(ParseError::MissingRequiredArguments(missing_args));
            }
        }
        Ok(())
    }
    // ------------------------------------------------------------
    // Boolean Utils
    // ------------------------------------------------------------
    fn is_option_token(token: &str) -> bool {
        token.starts_with('-') && token != "-"
    }
    // ------------------------------------------------------------
    // Initialization Utils
    // ------------------------------------------------------------
    fn initialize_parsed_args(template_args: &Vec<CliArgument>) -> ParsedArgs {
        let mut parsed_args: ParsedArgs = HashMap::new();
        for arg in template_args {
            match arg.variadic {
                true => parsed_args.insert(arg.name.clone(), Box::new(None::<Vec<String>>)),
                false => parsed_args.insert(arg.name.clone(), Box::new(None::<String>)),
            };
        }
        parsed_args
    }
    fn initialize_parsed_opts(template_opts: &Vec<CliOption>) -> ParsedOpts {
        let mut parsed_opts: ParsedOpts = HashMap::new();
        for opt in template_opts {
            match opt.args.len() < 2 {
                true => {
                    parsed_opts.insert(opt.name.clone(), Box::new(None::<String>));
                }
                false => {
                    if opt.optional {
                        let parsed_opt_args = Self::initialize_parsed_args(&opt.args);
                        parsed_opts.insert(opt.name.clone(), Box::new(parsed_opt_args));
                    } else {
                        // We don't initialize the parsed args,
                        // since we need to check if they are missing later
                        parsed_opts.insert(
                            opt.name.clone(),
                            Box::new(None::<HashMap<String, Box<dyn Any>>>),
                        );
                    }
                }
            };
        }
        parsed_opts
    }
}
