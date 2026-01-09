use crate::{
    Cli,
    error::{ParseError, suggest_similar::suggest_similar},
    types::CliOptionFlags,
};

impl Cli {
    pub fn handle_parse_error(&self, e: ParseError) {
        println!();
        match e {
            ParseError::InvalidCommand(command) => {
                eprintln!("error: Invalid command: {}\n", command);

                println!(
                    "tip: Available commands: {}",
                    self.commands
                        .iter()
                        .map(|cmd| cmd.name.clone())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                println!(
                    "( For more help on commands run: {} help )",
                    self.executable_name
                );
            }
            ParseError::TooManyArguments(args) => {
                eprintln!(
                    "error: Arguments: {:?} exceeded the maximum number of arguments\n",
                    args
                );

                let used_command = self.used_command.as_ref().unwrap();
                let command_def = self
                    .commands
                    .iter()
                    .find(|cmd| &cmd.name == used_command)
                    .unwrap();

                println!(
                    "tip: Arguments for '{}' command are: {}",
                    used_command,
                    command_def
                        .arguments
                        .iter()
                        .map(|arg| arg.reconstruct_name())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                println!(
                    "( For more help on arguments run: {} help {} )",
                    self.executable_name, used_command
                );
            }
            ParseError::MissingRequiredArguments(args) => {
                eprintln!("error: Missing required arguments: {:?}\n", args.join(" "));

                let used_command = self.used_command.as_ref().unwrap();
                let command_def = self
                    .commands
                    .iter()
                    .find(|cmd| &cmd.name == used_command)
                    .unwrap();

                println!(
                    "tip: Arguments for '{}' command are: {}",
                    used_command,
                    command_def
                        .arguments
                        .iter()
                        .map(|arg| arg.reconstruct_name())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                println!(
                    "( For more help on required arguments run: {} help {} )",
                    self.executable_name, used_command
                );
            }
            ParseError::MissingRequiredOptions(opts) => {
                eprintln!("error: Missing required options: {:?}\n", opts.join(", "));

                let used_command = self.used_command.as_ref().unwrap();
                let command_def = self
                    .commands
                    .iter()
                    .find(|cmd| &cmd.name == used_command)
                    .unwrap();

                println!(
                    "tip: Options for '{}' command are: {}",
                    used_command,
                    command_def
                        .options
                        .iter()
                        .map(|opt| format!("({})", opt.flags.to_string()))
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                println!(
                    "( For more help on options run: {} help {} )",
                    self.executable_name, used_command
                );
            }
            ParseError::MissingRequiredArgumentsForOption(idx, args) => {
                eprintln!(
                    "error: Missing required arguments for option: {:?}\n",
                    args.join(" ")
                );

                let used_command = self.used_command.as_ref().unwrap();
                let command_def = self
                    .commands
                    .iter()
                    .find(|cmd| &cmd.name == used_command)
                    .unwrap();

                let opt_def = &command_def.options[idx];

                println!(
                    "tip: Option '{}' is defined as: {} {}",
                    used_command,
                    opt_def.flags.to_string(),
                    opt_def
                        .args
                        .iter()
                        .map(|arg| arg.reconstruct_name())
                        .collect::<Vec<String>>()
                        .join(" ")
                );
                println!(
                    "( For more help on option arguments run: {} help {} )",
                    self.executable_name, used_command
                );
            }
            ParseError::InvalidOptionFlag(flag) => {
                eprintln!("error: Invalid option flag: {:?}\n", flag);

                let used_command = self.used_command.as_ref().unwrap();
                let command_def = self
                    .commands
                    .iter()
                    .find(|cmd| &cmd.name == used_command)
                    .unwrap();

                let mut long_flags = Vec::new();
                let mut short_flags = Vec::new();

                for opt in command_def.options.iter() {
                    match &opt.flags {
                        CliOptionFlags::Short(s) => {
                            short_flags.push(format!("-{}", s));
                        }
                        CliOptionFlags::Long(l) => {
                            long_flags.push(format!("--{}", l));
                        }
                        CliOptionFlags::ShortAndLong(s, l) => {
                            short_flags.push(format!("-{}", s));
                            long_flags.push(format!("--{}", l));
                        }
                    }
                }

                if flag.starts_with("--") {
                    println!("tip: {}", suggest_similar(flag, long_flags));
                } else {
                    println!(
                        "tip: Available short flags for `{}` are: {}",
                        used_command,
                        short_flags.join(", ")
                    );
                }
                println!(
                    "( For more help on options run: `{} help {}` )",
                    self.executable_name,
                    self.used_command.as_ref().unwrap()
                );
            }
        }
        println!();
    }
}
