use crate::{
    argument::{CliArgument, ParsedArg, ParsedArgs, ParsedOption, ParsedOptions},
    command::CliCommand,
    error::{CliError, UserError},
    help,
    option::CliOption,
    validate::validate_arguments_definition,
};
use std::{borrow::Cow, collections::HashMap};

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
            parsed_args: HashMap::new(),
            parsed_options: HashMap::new(),
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
        // ------- Check if the defined arguments are valid ---------
        {
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
        }

        // ------- Retrieve the arguments and options to match against ----------
        let command = if let Some(segment) = env_args.first() {
            self.commands
                .iter()
                .find(|command| command.name == Cow::Borrowed(segment))
        } else {
            None
        };
        let mut env_args = env_args.into_iter();
        let (args, options) = match command {
            Some(command) => {
                // Skip the command name
                env_args.next();
                (&command.arguments, &command.options)
            }
            None => {
                // If top level cli isn t configured, user needs help
                if self.arguments.is_empty() && self.options.is_empty() {
                    self.help();
                }

                // Use the top level arguments and options
                (&self.arguments, &self.options)
            }
        };
        let env_args = env_args.collect::<Vec<String>>();

        // ------- Check if all required options are provided by the user ---------
        let missing_options = options
            .iter()
            .filter(|option| option.required)
            .filter(|required_option| {
                let long_flag = &required_option.long_flag;
                let short_flag = &required_option.short_flag;

                if let Some(long_flag) = long_flag {
                    if env_args.contains(&long_flag.to_string()) {
                        return false;
                    }
                }
                if let Some(short_flag) = short_flag {
                    if env_args.contains(&short_flag.to_string()) {
                        return false;
                    }
                }
                true
            })
            .map(|option| option.name.to_string())
            .collect::<Vec<_>>();
        if !missing_options.is_empty() {
            return Err(CliError::UserError(UserError::MissingRequiredOptions(
                missing_options,
            )));
        }

        // --------------------------- Parsing ---------------------------
        // Initialize parsed arguments
        let mut parsed_args = HashMap::with_capacity(args.len());
        for argument in args {
            let multiple = if argument.multiple {
                ParsedArg::Multiple(None)
            } else {
                ParsedArg::Single(None)
            };
            parsed_args.insert(argument.name.to_string(), multiple);
        }

        // Initialize parsed options
        let mut parsed_options = HashMap::with_capacity(options.len());
        for option in options {
            let parsed_option = if option.arguments.is_empty() {
                ParsedOption::Boolean(false)
            } else {
                let mut args = HashMap::with_capacity(option.arguments.len());
                for argument in &option.arguments {
                    let multiple = if argument.multiple {
                        ParsedArg::Multiple(None)
                    } else {
                        ParsedArg::Single(None)
                    };
                    args.insert(argument.name.to_string(), multiple);
                }
                ParsedOption::Args(args)
            };
            parsed_options.insert(option.name.to_string(), parsed_option);
        }

        // Parse
        for segment in env_args {
            todo!();
        }

        // --------------------------- Finalize ---------------------------
        // Set the parsed data
        self.parsed_args = parsed_args.clone();
        self.parsed_options = parsed_options.clone();

        // Run actions
        match command {
            Some(command) => {
                if let Some(action) = &command.action {
                    action(parsed_args, parsed_options);
                }
            }
            None => {
                if let Some(action) = &self.action {
                    action(parsed_args, parsed_options);
                }
            }
        }

        Ok(())
    }

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    // User logic
    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-
    pub fn help(&self) {
        let executable_name = std::env::args().next().unwrap_or_else(|| "cli".to_string());

        println!(
            "Usage: {}\n",
            help::usage_string(&executable_name, &self.arguments, &self.options, None)
        );

        if !self.arguments.is_empty() {
            println!("Arguments: \n{}\n", help::arguments_list(&self.arguments));
        }
        if !self.options.is_empty() {
            println!("Options: \n{}\n", help::options_list(&self.options));
        }
        if !self.commands.is_empty() {
            println!("Commands: \n{}\n", help::commands_list(&self.commands));
            println!(
                "For info on a specific command, use: {} help [command]\n",
                executable_name
            );
        }
        std::process::exit(0);
    }
}
