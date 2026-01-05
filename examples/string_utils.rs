use dsl_cli::cli;

fn main() {
    cli!(
        // These 3 fields are required
        name "string_utils",
        version "0.1.0",
        description "A simple CLI for string utilities",

        // Commands are defined as: cmd <name> ["description"] { ... }
        cmd reverse "Reverse a string" {
            arg string "The string to reverse" String,
        },

        cmd split "Split a string into words" {
            arg string "The string to split" String,
            opt "-s, --separator" "The separator to use" {
                arg separator "The separator to use" String,
            },
        },
    );

    match parsed {
        Command::Reverse(args, _) => {
            println!(
                "Reversed string: {}",
                args.string.chars().rev().collect::<String>()
            );
        }
        Command::Split(args, opts) => {
            println!(
                "Split string: {}",
                args.string
                    .split(&opts.separator)
                    .collect::<Vec<&str>>()
                    .join(" ")
            );
        }
    }
}
