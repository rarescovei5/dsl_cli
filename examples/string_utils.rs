use commander::Cli;

fn main() {
    let mut cli_program = Cli::new();

    // Instance details
    cli_program
        .name("string-util")
        .version("0.1.0")
        .description("A simple rust cli that exposes string utilities");

    // Separate command
    cli_program
        .command("split")
        .description("Split a string into substrings and display as an array")
        .argument_with_description("<string>", "The String to split")
        .option_with_description("-s, --separator <char>", "The Delimiter to Use")
        .action(|args, options| {
            let string_to_split = args.get("string").unwrap();
            let separator = options.get("--separator").get("char").unwrap();
            // split the string by the separator
            println!(
                "{:?}",
                string_to_split.split(&separator).collect::<Vec<&str>>()
            );
        });

    cli_program.parse();
}
