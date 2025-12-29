use std::borrow::Cow;

use crate::{argument::CliArgument, command::CliCommand, option::CliOption};

const PADDING: usize = 4;

/// Calculate the width of option left cells (flags + arguments)
pub fn options_width(options: &[CliOption<'_>]) -> usize {
    options
        .iter()
        .map(|opt| {
            let flags = [opt.short_flag.as_ref(), opt.long_flag.as_ref()]
                .iter()
                .filter_map(|flag| flag.as_ref().map(|f| f.to_string()))
                .collect::<Vec<String>>()
                .join(", ");

            let args = opt
                .arguments
                .iter()
                .map(|a| reconstruct_arg_string(a))
                .collect::<Vec<String>>();

            if args.is_empty() {
                flags.len()
            } else {
                flags.len() + 1 + args.join(" ").len()
            }
        })
        .max()
        .unwrap_or(0)
}

/// Calculate the width of argument left cells
pub fn arguments_width(arguments: &[CliArgument<'_>]) -> usize {
    arguments
        .iter()
        .map(|arg| reconstruct_arg_string(arg).len())
        .max()
        .unwrap_or(0)
}

pub fn options_list(options: &Vec<CliOption<'_>>, min_width: usize) -> String {
    if options.is_empty() {
        return String::new();
    }

    let left_cells: Vec<String> = options
        .iter()
        .map(|opt| {
            let flags = [opt.short_flag.as_ref(), opt.long_flag.as_ref()]
                .iter()
                .filter_map(|flag| flag.as_ref().map(|f| f.to_string()))
                .collect::<Vec<String>>()
                .join(", ");

            let args = opt
                .arguments
                .iter()
                .map(|a| reconstruct_arg_string(a))
                .collect::<Vec<String>>();

            if args.is_empty() {
                flags
            } else {
                format!("{} {}", flags, args.join(" "))
            }
        })
        .collect();

    let left_width = left_cells
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0)
        .max(min_width);

    left_cells
        .into_iter()
        .zip(options.iter())
        .map(|(left, opt)| {
            let desc = opt.description.as_deref().unwrap_or("");
            let pad = left_width.saturating_sub(left.len()) + PADDING;
            format!("  {}{}{}", left, " ".repeat(pad), desc)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn arguments_list(arguments: &Vec<CliArgument<'_>>, min_width: usize) -> String {
    if arguments.is_empty() {
        return String::new();
    }

    let left_cells: Vec<String> = arguments
        .iter()
        .map(|arg| reconstruct_arg_string(arg))
        .collect();

    let left_width = left_cells
        .iter()
        .map(|s| s.len())
        .max()
        .unwrap_or(0)
        .max(min_width);

    left_cells
        .into_iter()
        .zip(arguments.iter())
        .map(|(left, arg)| {
            let desc = arg.description.as_deref().unwrap_or("");
            // Minimum 2 spaces between the left column and description.
            let pad = left_width.saturating_sub(left.len()) + PADDING;
            format!("  {}{}{}", left, " ".repeat(pad), desc)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn commands_list(commands: &Vec<CliCommand<'_>>) -> String {
    if commands.is_empty() {
        return String::new();
    }

    let left_cells: Vec<String> = commands.iter().map(|c| c.name.to_string()).collect();
    let left_width = left_cells.iter().map(|s| s.len()).max().unwrap_or(0);

    left_cells
        .into_iter()
        .zip(commands.iter())
        .map(|(left, cmd)| {
            let desc = cmd.description.as_deref().unwrap_or("");
            let pad = left_width.saturating_sub(left.len()) + PADDING;
            format!("  {}{}{}", left, " ".repeat(pad), desc)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn usage_string(
    arguments: &Vec<CliArgument<'_>>,
    options: &Vec<CliOption<'_>>,
    command_name: Option<&Cow<'_, str>>,
) -> String {
    // Build usage string
    let mut usage_parts = Vec::new();
    // Add command name if it exists
    if let Some(command_name) = command_name {
        usage_parts.push(command_name.to_string());
    }
    // Add options
    if !options.is_empty() {
        usage_parts.push("[options]".to_string());
    }
    // Add arguments
    usage_parts.extend(arguments.iter().map(|arg| reconstruct_arg_string(arg)));
    // Return usage string
    usage_parts.join(" ")
}

pub fn reconstruct_arg_string(arg: &CliArgument<'_>) -> String {
    let mut name = arg.name.to_string();
    if arg.multiple {
        name += "...";
    }
    if arg.required {
        name = "<".to_owned() + &name + ">";
    } else {
        name = "[".to_owned() + &name + "]";
    }
    name
}
