use std::{any::Any, collections::HashMap};

use crate::parse::{
    ParseError, TemplateArgs, TemplateOpts, initialize_parsed_args, initialize_parsed_opts,
};

pub type ParsedArgs = HashMap<String, Option<Box<dyn Any>>>;
pub type ParsedOpts = HashMap<String, Option<Box<dyn Any>>>;

pub struct Parser;

impl Parser {
    pub fn parse_args(
        env_args: Vec<String>,
        template_args: TemplateArgs,
        template_opts: TemplateOpts,
    ) -> Result<(ParsedArgs, ParsedOpts), ParseError> {
        let mut parsed_args = initialize_parsed_args(&template_args);
        let mut parsed_opts = initialize_parsed_opts(&template_opts);
        let mut tokens = env_args.into_iter().peekable();
        let mut positional_idx = 0;

        while let Some(token) = tokens.next() {
            if Self::is_option_token(&token) {
                // Check if the option is included in the template
                if !template_opts.iter().any(|opt| opt.flags() == token) {
                    return Err(ParseError::InvalidOptionFlag(token));
                }

                let opt_def = template_opts
                    .iter()
                    .find(|opt| opt.flags() == token)
                    .unwrap();

                // Option has no arguments = flag-only option
                if opt_def.args().is_empty() {
                    parsed_opts.insert(opt_def.name(), Some(Box::new(true)));
                    continue;
                }

                // Handle positional arguments for option
                let opt_args = opt_def.args();
                let mut parsed_opt_args = initialize_parsed_args(&opt_args);
                let mut idx = 0;

                while idx < parsed_opt_args.len() {
                    if tokens.peek().is_none() || Self::is_option_token(tokens.peek().unwrap()) {
                        break;
                    }

                    let arg_def = &opt_args[idx];
                    if arg_def.variadic() {
                        let mut values = vec![tokens.next().unwrap()];
                        while tokens.peek().is_some()
                            && !Self::is_option_token(tokens.peek().unwrap())
                        {
                            values.push(tokens.next().unwrap());
                        }
                        parsed_opt_args.insert(arg_def.name(), Some(Box::new(values)));
                        idx += 1;
                    } else {
                        parsed_opt_args
                            .insert(arg_def.name(), Some(Box::new(tokens.next().unwrap())));
                        idx += 1;
                    }
                }

                Self::check_for_missing_required_args(&opt_args, idx, true)?;

                parsed_opts.insert(opt_def.name(), Some(Box::new(parsed_opt_args)));
            } else {
                // Check if we've gone past the number of positional arguments
                if positional_idx >= template_args.len() {
                    let mut remaining_args = vec![token];
                    remaining_args.extend(tokens);
                    return Err(ParseError::TooManyArguments(remaining_args));
                }

                // Handle positional arguments
                let arg_def = &template_args[positional_idx];
                if arg_def.variadic() {
                    let mut values = vec![token];
                    while tokens.peek().is_some() && !Self::is_option_token(tokens.peek().unwrap())
                    {
                        values.push(tokens.next().unwrap());
                    }
                    parsed_args.insert(arg_def.name(), Some(Box::new(values)));
                    positional_idx += 1;
                } else {
                    parsed_args.insert(arg_def.name(), Some(Box::new(token)));
                    positional_idx += 1;
                }
            }
        }

        Self::check_for_missing_required_args(&template_args, positional_idx, false)?;
        Self::check_for_missing_required_opts(&parsed_opts, &template_opts)?;

        Ok((parsed_args, parsed_opts))
    }

    // ------------------------------------------------------------
    // Validation methods
    // ------------------------------------------------------------
    fn check_for_missing_required_opts(
        parsed_opts: &ParsedOpts,
        template_opts: &TemplateOpts,
    ) -> Result<(), ParseError> {
        let mut missing_required_opts = Vec::new();
        for opt in template_opts.iter().filter(|opt| !opt.optional()) {
            if parsed_opts.get(&opt.name()).unwrap().is_none() {
                missing_required_opts.push(opt.name());
            }
        }
        if !missing_required_opts.is_empty() {
            return Err(ParseError::MissingRequiredOptions(missing_required_opts));
        }
        Ok(())
    }
    fn check_for_missing_required_args(
        template_args: &TemplateArgs,
        positional_idx: usize,
        are_opt_args: bool,
    ) -> Result<(), ParseError> {
        if positional_idx < template_args.iter().filter(|arg| !arg.optional()).count() {
            let missing_args = template_args[positional_idx..]
                .iter()
                .filter(|arg| !arg.optional())
                .map(|arg| arg.name())
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
        token.starts_with('-')
    }
}
