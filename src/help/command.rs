use crate::{
    help::utils::{arguments_list, arguments_width, options_list, options_width, usage_string},
    types::CliCommand,
};

impl<'a> CliCommand<'a> {
    pub(crate) fn help(&self) {
        println!("\nCommand: {}", self.name);

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

        let usage_string = usage_string(&self.arguments, &self.options, Some(&self.name));
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

        println!();

        std::process::exit(0);
    }
}
