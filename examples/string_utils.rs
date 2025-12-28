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
        .argument_with_description("<string3...>", "The String to split")
        .argument_with_description("<string2>", "The String to split")
        .option_with_description("-s, --separator <char>", "The Delimiter to Use")
        .action(|args, options| {
            dbg!(args);
            dbg!(options);
        });

    cli_program.parse();
}
