# Commander (Rust)

The complete solution for Rust command-line interfaces.

Inspired by [Commander.js](https://github.com/tj/commander.js), but built for Rust.

---

## Commander.js-style docs (for this Rust crate)

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Declaring program variable](#declaring-program-variable)
- [Options](#options)
  - [Common option types, boolean and value](#common-option-types-boolean-and-value)
  - [Default option value](#default-option-value)
  - [Required option](#required-option)
  - [Variadic option](#variadic-option)
  - [Version option](#version-option)
  - [More configuration](#more-configuration)
  - [Custom option processing](#custom-option-processing)
- [Commands](#commands)
  - [Command-arguments](#command-arguments)
    - [More configuration](#more-configuration-1)
    - [Custom argument processing](#custom-argument-processing)
  - [Action handler](#action-handler)
- [Automated help](#automated-help)
  - [.parse() and .try_parse()](#parse-and-try_parse)
  - [Parsing Configuration](#parsing-configuration)
  - [Type conversion](#type-conversion)

---

## Installation

Add the crate to your project:

```toml
[dependencies]
commander = "0.1.0"
```

Or with Cargo:

```bash
cargo add commander
```

## Quick Start

Here’s a complete example (from `examples/string_utils.rs`, adapted):

```rust
use commander::Cli;

fn main() {
    let mut cli_program = Cli::new();

    cli_program
        .name("string-util")
        .version("0.1.0")
        .description("A simple rust cli that exposes string utilities");

    cli_program
        .command("split")
        .description("Split a string into substrings and display as an array")
        .argument_with_description("<string>", "The String to split")
        .option_with_description("-s, --separator <char>", "The Delimiter to Use")
        .action(|args, options| {
            let string_to_split = args.get("string").unwrap();
            let separator = options.get("--separator").get("char").unwrap();
            println!(
                "{:?}",
                string_to_split.split(&separator).collect::<Vec<&str>>()
            );
        });

    cli_program.parse();
}
```

Try it:

```bash
cargo run --example str -- split a,b,c -s ,
```

And built-in help:

```bash
cargo run --example str -- help
cargo run --example str -- help split
```

## Declaring program variable

The main entry point is `Cli`:

```rust
use commander::Cli;

let mut program = Cli::new();
```

Then configure it with fluent builder methods like `.name(..)`, `.version(..)`, `.description(..)`, etc.

## Options

Options are defined by a flag string and optional description.

- `-s` is a short flag
- `--separator` is a long flag
- option “arguments” are declared using the same syntax as positional arguments:
  - `<value>` required
  - `[value]` optional
  - `...` for variadic (multiple values)

### Common option types, boolean and value

**Boolean flag (present/absent):**

```rust
program.option_with_description("-v, --verbose", "Enable verbose output");
```

Read it:

```rust
let verbose = options.get("--verbose").as_bool();
```

**Value option:**

```rust
program.option_with_description("-o, --output <file>", "Output file");
```

Read it:

```rust
let output_path = options.get("--output").get("file").unwrap();
```

### Default option value

Use `unwrap_or` to provide a default:

```rust
let separator = options
    .get("--separator")
    .get("char")
    .unwrap_or(",");
```

### Required option

Use `.required_option(..)` / `.required_option_with_description(..)`:

```rust
program.required_option_with_description("-c, --count <n>", "How many times");
```

If the user does not supply it, parsing fails with a user-facing error (and `parse()` exits with status code 1).

### Variadic option

Use `...` to accept multiple values:

```rust
program.option_with_description("--include <files...>", "Files to include");
```

Read it:

```rust
let files = options.get("--include").get("files").unwrap_vec();
```

### Version option

You can set version metadata:

```rust
program.version("0.1.0");
```

This version is displayed in the top-level help header. (There is no automatic `-V/--version` option)

### More configuration

Top-level configuration:

- `.name("my-cli")`
- `.description("...")`
- `.version("...")`

### Custom option processing

All parsed values come through as strings (or vector of strings). Convert them however you like:

```rust
let count: usize = options
    .get("--count")
    .get("n")
    .unwrap()
    .parse()
    .expect("count must be a number");
```

## Commands

Commands are subcommands attached to a `Cli` instance. Each command can have its own arguments, options, and action.

### Command-arguments

Add positional arguments to a command:

```rust
program
    .command("copy")
    .argument_with_description("<from>", "Source path")
    .argument_with_description("<to>", "Destination path");
```

Argument syntax:

- `<name>` required
- `[name]` optional
- `<name...>` variadic (multiple values)

#### More configuration

Per-command metadata:

```rust
program
    .command("split")
    .description("Split a string into substrings");
```

#### Custom argument processing

Like options, arguments are parsed as strings (or vector of strings). Convert them to your own types.

### Action handler

Provide an action closure which receives parsed `args` and `options`:

```rust
program
    .command("echo")
    .argument("<text>")
    .option("-n, --no-newline")
    .action(|args, options| {
        let text = args.get("text").unwrap();
        let no_newline = options.get("--no-newline").as_bool();
        if no_newline {
            print!("{text}");
        } else {
            println!("{text}");
        }
    });
```

## Automated help

This crate generates help output for:

- Top-level `Cli`
- Individual commands (`CliCommand`)

And supports the user-facing `help` command:

```bash
my-cli help
my-cli help <command>
```

### Parsing Configuration

The library validates definitions to prevent ambiguous CLIs:

- Required args cannot appear after optional args  
  Example of invalid definition: `[dir] <file>`
- Variadic args (`<files...>`) must come last

### Type conversion

Values are accessed via `.get(..)` which returns a `Value`:

- `.unwrap()` -> `String` (panics if missing)
- `.unwrap_or(default)` -> `String`
- `.unwrap_vec()` -> `Vec<String>`
- `.as_bool()` -> `bool` for boolean flags



