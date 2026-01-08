use crate::{Cli, CliCommand};

impl Cli {
    pub fn show_help(&self, command_name: String) {
        // This might not exist if the command name is cli. Otherwise it will be a valid command.
        let cmd_def = self.commands.iter().find(|cmd| cmd.name == command_name);

        let mut cmds_info = Vec::new();
        let mut args_info = Vec::new();
        let mut opts_info = Vec::new();

        match cmd_def {
            Some(cmd_def) => {
                args_info = cmd_def.args_info();
                opts_info = cmd_def.opts_info();
            }
            None => {}
        }

        if command_name == "cli" {
            cmds_info = self
                .commands
                .iter()
                .map(|cmd| cmd.info())
                .collect::<Vec<(String, String)>>();
        }

        let args_max_width = args_info
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0);
        let opts_max_width = opts_info
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0);
        let cmds_max_width = cmds_info
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0);
        let max_width = args_max_width.max(opts_max_width).max(cmds_max_width) + 2;

        // Display Usage
        match cmd_def {
            Some(cmd_def) => {
                println!();
                let mut usage_string = String::new();

                usage_string.push_str(&self.executable_name);

                if !args_info.is_empty() {
                    let args_string = " ".to_owned()
                        + &args_info
                            .iter()
                            .map(|(name, _)| name.as_str())
                            .collect::<Vec<&str>>()
                            .join(" ");
                    usage_string.push_str(&args_string);
                }

                if !opts_info.is_empty() {
                    usage_string.push_str(" [options]");
                }

                println!("Usage: {}", usage_string);
            }
            None => {}
        }
        // Display Description
        if command_name != "cli" {
            println!();
            let cmd_def = cmd_def.unwrap();

            let description = &cmd_def.description;

            if let Some(description) = description {
                println!("{}", description)
            } else {
                println!("No description available")
            }
        } else {
            println!();
            println!("{}", self.description);
        }
        // Display Commands
        if !cmds_info.is_empty() {
            println!();
            println!("Commands:");
            for (name, description) in cmds_info {
                let width = name.len();
                let padding = " ".repeat(max_width - width);
                println!("  {}{}{}", name, padding, description);
            }
        }
        // Display Arguments
        if !args_info.is_empty() {
            println!();
            println!("Arguments:");
            for (name, description) in args_info {
                let width = name.len();
                let padding = " ".repeat(max_width - width);
                println!("  {}{}{}", name, padding, description);
            }
        }
        // Display Options
        if !opts_info.is_empty() {
            println!();
            println!("Options:");
            for (name, description) in opts_info {
                let width = name.len();
                let padding = " ".repeat(max_width - width);
                println!("  {}{}{}", name, padding, description);
            }
        }
        println!();
    }
}
