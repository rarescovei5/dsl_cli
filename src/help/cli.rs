use crate::{
    Cli,
    help::utils::{
        arguments_list, arguments_width, commands_list, options_list, options_width, usage_string,
    },
};
use std::borrow::Cow;

impl<'a> Cli<'a> {
    pub(crate) fn help(&self) {
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

        let usage_string = usage_string(&self.arguments, &self.options, None);
        if !usage_string.is_empty() {
            println!("\nUsage: {} {}", executable_name, usage_string);
        }

        let column_width = arguments_width(&self.arguments).max(options_width(&self.options));

        if !self.arguments.is_empty() {
            println!(
                "\nArguments: \n{}",
                arguments_list(&self.arguments, column_width)
            );
        }
        if !self.options.is_empty() {
            println!("\nOptions: \n{}", options_list(&self.options, column_width));
        }
        if !self.commands.is_empty() {
            println!("\nCommands: \n{}", commands_list(&self.commands));
            println!(
                "\nFor info on a specific command, use: {} help [command]",
                executable_name
            );
        }

        println!();

        std::process::exit(0);
    }
}
