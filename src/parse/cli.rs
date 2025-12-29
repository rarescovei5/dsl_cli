use std::borrow::Cow;

use crate::{
    error::{CliError, UserError},
    parse::{
        types::{ParsedArg, ParsedArgs, ParsedOption, ParsedOptions},
        utils::{
            TokenStream, find_option, parse_option_args, validate_arguments_definition,
            validate_no_unknown_options, validate_required_options,
        },
    },
    types::{Cli, CliArgument, CliOption},
    utils::reconstruct_arg_string,
};

impl<'a> Cli<'a> {
    /// Parse the command line arguments
    pub fn parse(&mut self) {
        let args = std::env::args().skip(1).collect::<Vec<String>>();

        self.parse_args(args)
    }
    /// Parse the given environment arguments
    pub fn parse_args(&mut self, env_args: Vec<String>) {
        match self.try_parse(env_args) {
            Ok(()) => (),
            Err(error) => {
                println!("{}", error);
                std::process::exit(1);
            }
        }
    }

    // Parse Implementation
    fn try_parse(&mut self, env_args: Vec<String>) -> Result<(), CliError> {
        // Validate CLI definition (developer errors)
        self.validate_definition()?;

        //  Resolve command context (which args/options to match against)
        let (command_idx, tokens) = self.resolve_command_context(env_args);

        // Validate user input and parse tokens
        let (parsed_args, parsed_options) = {
            let (args, options) = match command_idx {
                Some(idx) => (&self.commands[idx].arguments, &self.commands[idx].options),
                None => (&self.arguments, &self.options),
            };

            // Validate user input (user errors)
            validate_required_options(options, &tokens)?;
            validate_no_unknown_options(options, &tokens)?;

            // Parse tokens into structured data
            Self::parse_tokens(tokens, args, options)?
        };

        // Finalize (store results and run action)
        self.parsed_args = parsed_args.clone();
        self.parsed_options = parsed_options.clone();

        // Run action
        let action = match command_idx {
            Some(idx) => &self.commands[idx].action,
            None => &self.action,
        };
        if let Some(action) = action {
            action(parsed_args, parsed_options);
        }

        Ok(())
    }

    /// Validate that the CLI definition is correct (no invalid argument orderings, etc.)
    fn validate_definition(&self) -> Result<(), CliError> {
        validate_arguments_definition(&self.arguments)?;
        for option in &self.options {
            validate_arguments_definition(&option.arguments)?;
        }
        for command in &self.commands {
            validate_arguments_definition(&command.arguments)?;
            for option in &command.options {
                validate_arguments_definition(&option.arguments)?;
            }
        }
        Ok(())
    }

    /// Determine which command (if any) is being invoked, and return its index + remaining tokens.
    fn resolve_command_context(&self, env_args: Vec<String>) -> (Option<usize>, Vec<String>) {
        let first_arg = env_args.first();

        if first_arg.is_some() && first_arg.unwrap() == "help" {
            let second_arg = env_args.get(1);

            if second_arg.is_some() {
                let second_arg = second_arg.unwrap();
                let command_idx = self
                    .commands
                    .iter()
                    .position(|cmd| cmd.name == Cow::Borrowed(second_arg.as_str()));

                if command_idx.is_some() {
                    self.commands[command_idx.unwrap()].help();
                } else {
                    self.help();
                }
            } else {
                self.help();
            }
        }

        let command_idx = first_arg.and_then(|first| {
            self.commands
                .iter()
                .position(|cmd| cmd.name == Cow::Borrowed(first.as_str()))
        });

        let tokens = if command_idx.is_some() {
            env_args.into_iter().skip(1).collect()
        } else {
            // Show help if top-level CLI has no args/options configured
            if self.arguments.is_empty() && self.options.is_empty() {
                self.help();
            }
            env_args
        };

        (command_idx, tokens)
    }

    /// Parse the token stream into ParsedArgs and ParsedOptions.
    fn parse_tokens(
        tokens: Vec<String>,
        args: &[CliArgument<'_>],
        options: &[CliOption<'_>],
    ) -> Result<(ParsedArgs, ParsedOptions), CliError> {
        let mut parsed_args = init_parsed_args(args);
        let mut parsed_options = init_parsed_options(options);
        let mut stream = TokenStream::new(tokens);
        let mut positional_idx = 0;

        while let Some(token) = stream.next() {
            if let Some(option) = find_option(&token, options) {
                // Parse option and its arguments
                parse_option_args(&mut stream, option, &mut parsed_options)?;
            } else {
                // It's a positional argument
                if positional_idx < args.len() {
                    let arg_def = &args[positional_idx];
                    if arg_def.variadic {
                        // Variadic: this token + all remaining non-option tokens
                        let mut values = vec![token];
                        while !stream.peek_is_option() && stream.peek().is_some() {
                            values.push(stream.next().unwrap());
                        }
                        parsed_args
                            .insert(arg_def.name.to_string(), ParsedArg::Multiple(Some(values)));
                        positional_idx += 1;
                    } else {
                        parsed_args
                            .insert(arg_def.name.to_string(), ParsedArg::Single(Some(token)));
                        positional_idx += 1;
                    }
                } else {
                    let remaining_args = stream.remaining_args();

                    return Err(CliError::UserError(UserError::TooManyArguments(
                        remaining_args,
                    )));
                }
            }
        }

        // Final check: user must supply all required arguments
        let required_args_count = args.iter().filter(|arg| arg.required).count();
        if positional_idx < required_args_count {
            let missing_args = args[positional_idx..required_args_count]
                .iter()
                .map(|arg| reconstruct_arg_string(arg))
                .collect::<Vec<String>>();
            return Err(CliError::UserError(UserError::MissingRequiredArguments(
                missing_args,
            )));
        }

        Ok((parsed_args, parsed_options))
    }
}

pub fn init_parsed_args(args: &[CliArgument<'_>]) -> ParsedArgs {
    let mut parsed = ParsedArgs::new();
    for arg in args {
        let value = if arg.variadic {
            ParsedArg::Multiple(None)
        } else {
            ParsedArg::Single(None)
        };
        parsed.insert(arg.name.to_string(), value);
    }
    parsed
}

pub fn init_parsed_options(options: &[CliOption<'_>]) -> ParsedOptions {
    let mut parsed = ParsedOptions::new();
    for opt in options {
        let value = if opt.arguments.is_empty() {
            ParsedOption::Boolean(false)
        } else {
            ParsedOption::Args(init_parsed_args(&opt.arguments))
        };
        parsed.insert(opt.name.to_string(), value);
    }
    parsed
}
