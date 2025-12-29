use crate::{
    argument::CliArgument,
    command::CliCommand,
    error::{CliError, UserError},
    help::{self, reconstruct_arg_string},
    option::CliOption,
    parse::{
        ParsedArg, ParsedArgs, ParsedOptions, TokenStream, find_option, init_parsed_args,
        init_parsed_options, parse_option_args, validate_no_unknown_options,
        validate_required_options,
    },
    validate::validate_arguments_definition,
};
use std::borrow::Cow;

pub struct Cli<'a> {
    // Instance
    name: Option<Cow<'a, str>>,
    description: Option<Cow<'a, str>>,
    version: Option<Cow<'a, str>>,
    // Logic
    commands: Vec<CliCommand<'a>>,
    options: Vec<CliOption<'a>>,
    arguments: Vec<CliArgument<'a>>,
    action: Option<Box<dyn Fn(ParsedArgs, ParsedOptions) + 'a>>,
    // Parsed data
    pub parsed_args: ParsedArgs,
    pub parsed_options: ParsedOptions,
}

impl<'a> Cli<'a> {
    /// Create a new Cli instance
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            version: None,
            commands: Vec::new(),
            options: Vec::new(),
            arguments: Vec::new(),
            action: None,
            parsed_args: ParsedArgs::new(),
            parsed_options: ParsedOptions::new(),
        }
    }

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Cli Instance Initialization
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    /// Set the name of the Cli instance
    pub fn name<T>(&mut self, name: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.name = Some(name.into());
        self
    }
    /// Set the description of the Cli instance
    pub fn description<T>(&mut self, description: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.description = Some(description.into());
        self
    }
    /// Set the version of the Cli instance
    pub fn version<T>(&mut self, version: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.version = Some(version.into());
        self
    }

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Arguments Logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    /// Add a new argument (without a description) to the CliCommand
    pub fn argument<T>(&mut self, name: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.arguments.push(CliArgument::new(name, None));
        self
    }
    /// Add a new argument (with a description) to the CliCommand
    pub fn argument_with_description<T>(&mut self, name: T, description: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.arguments
            .push(CliArgument::new(name, Some(description)));
        self
    }
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Options Logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    /// Add a new option (without a description) to the CliCommand
    pub fn option<T>(&mut self, name: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options.push(CliOption::new(name, None, false));
        self
    }
    /// Add a new option (with a description) to the CliCommand
    pub fn option_with_description<T>(&mut self, name: T, description: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options
            .push(CliOption::new(name, Some(description), false));
        self
    }
    /// Add a new required option (without a description) to the CliCommand
    pub fn required_option<T>(&mut self, name: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options.push(CliOption::new(name, None, true));
        self
    }
    /// Add a new required option (with a description) to the CliCommand
    pub fn required_option_with_description<T>(&mut self, name: T, description: T) -> &mut Self
    where
        T: Into<Cow<'a, str>>,
    {
        self.options
            .push(CliOption::new(name, Some(description), true));
        self
    }
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Action Logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    /// Add an action to be called with the parsed arguments and options
    pub fn action<F>(&mut self, action: F) -> &mut Self
    where
        F: Fn(ParsedArgs, ParsedOptions) + 'a,
    {
        self.action = Some(Box::new(action));
        self
    }

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Command Logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    /// Add a new command to the Cli instance
    pub fn command<T>(&mut self, name: T) -> &mut CliCommand<'a>
    where
        T: Into<Cow<'a, str>>,
    {
        let new_command = CliCommand::new(name);
        self.commands.push(new_command);
        self.commands.last_mut().unwrap()
    }

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // Parse logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
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
    pub fn try_parse(&mut self, env_args: Vec<String>) -> Result<(), CliError> {
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
                    if arg_def.multiple {
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

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // User logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    pub fn help(&self) {
        if let Some(name) = &self.name {
            let version = "v".to_string() + self.version.as_ref().unwrap_or(&Cow::Borrowed(""));
            println!("\n{} {}", name, version);
        }

        // Print description
        if let Some(description) = &self.description {
            println!("\n{}", description);
        }

        let executable_name = std::env::args()
            .next()
            .and_then(|path| {
                std::path::Path::new(&path)
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| "cli".to_string());

        let usage_string = help::usage_string(&self.arguments, &self.options, None);
        if !usage_string.is_empty() {
            println!("\nUsage: {} {}", executable_name, usage_string);
        }

        let column_width =
            help::arguments_width(&self.arguments).max(help::options_width(&self.options));

        if !self.arguments.is_empty() {
            println!(
                "\nArguments: \n{}",
                help::arguments_list(&self.arguments, column_width)
            );
        }
        if !self.options.is_empty() {
            println!(
                "\nOptions: \n{}",
                help::options_list(&self.options, column_width)
            );
        }
        if !self.commands.is_empty() {
            println!("\nCommands: \n{}", help::commands_list(&self.commands));
            println!(
                "\nFor info on a specific command, use: {} help [command]",
                executable_name
            );
        }

        println!();

        std::process::exit(0);
    }
}
