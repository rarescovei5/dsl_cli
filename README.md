# `dsl_cli`

A small **proc-macro DSL** for defining command-line interfaces in Rust and getting **typed** parsed results back (via a generated `Command` enum + `Args/Opts` structs).

---

## What you write

You describe your CLI in a single macro invocation:

- **Top-level metadata**: `name`, `version`, `description` (required)
- **Commands**: `cmd <name> ["description"] { ... }`
- **Arguments**: `arg <name> ["description"] [: Type] [= <default?>],`
- **Options**:
  - `opt "<flags>" ["description"] { ... }` (optional)
  - `req_opt "<flags>" ["description"] { ... }` (required)

The macro generates:

- **`Command` enum** with one variant per command
- **`<Cmd>Args` / `<Cmd>Opts` structs** (typed)
- A **`parsed` variable** you can `match` on inside `main`

---

## Quick start (example in this repo)

This is `examples/string_utils.rs`:

```rust
use dsl_cli::cli;

fn main() {
    cli!(
        name "string_utils",
        version "0.1.0",
        description "A simple CLI for string utilities",

        cmd reverse "Reverse a string" {
            arg string "The string to reverse": String,
        },

        cmd split "Split a string into words" {
            arg string "The string to split": String,
            opt "-s, --separator" "The separator to use" {
                arg separator "The separator to use": String,
            },
        },
    );

    match parsed {
        Command::Reverse(args, _) => {
            println!("{}", args.string.chars().rev().collect::<String>());
        }
        Command::Split(args, opts) => {
            println!("{}", args.string.split(&opts.separator).collect::<Vec<_>>().join(" "));
        }
    }
}
```

Run it:

```bash
cargo run --example string_utils -- reverse hello
cargo run --example string_utils -- split "a,b,c" -s ","
```

---

## Types & parsing rules

- **Type conversion**: values are parsed with `FromStr` (`.parse().unwrap()` in generated code).
- **Optional positional args**: use `Option<T>`.
- **Variadic positional args**: use `Vec<T>` (or `Option<Vec<T>>`).
- **Defaults**: only allowed for optional types: `Option<T> = <expr>` (the generated field becomes `T`).
- **Boolean flags**: an `opt`/`req_opt` with *no* `arg` block becomes `bool`.
- **Option flags**: `"-s, --separator"` supports short and/or long.
---

