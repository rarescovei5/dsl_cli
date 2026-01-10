use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_macro_input;

mod utils;
use crate::utils::*;

mod ast;
use crate::ast::*;

mod type_gen;
use crate::type_gen::*;

mod cli_setup_gen;
use crate::cli_setup_gen::*;

mod from_parsed_impl;
use crate::from_parsed_impl::*;

#[proc_macro]
pub fn cli(input: TokenStream) -> TokenStream {
    let dsl = parse_macro_input!(input as CliDsl);

    // Generate CLI setup
    let cli_setup = generate_cli_setup(&dsl);

    // Generate structs for each command
    let args_structs: Vec<TokenStream2> = dsl
        .commands
        .iter()
        .map(|cmd| generate_args_struct(&cmd.arguments, &to_pascal_case(&cmd.name.to_string())))
        .collect();
    let opts_structs: Vec<TokenStream2> = dsl
        .commands
        .iter()
        .map(|cmd| generate_opts_struct(&cmd.options, &to_pascal_case(&cmd.name.to_string())))
        .collect();

    // Generate the enum for the commands
    let command_fields = generate_fields_for_command_enum(&dsl.commands);

    // Generate FromParsed implementations
    let args_from_parsed: Vec<TokenStream2> = dsl
        .commands
        .iter()
        .map(generate_from_parsed_impl_for_args)
        .collect();

    let opts_from_parsed: Vec<TokenStream2> = dsl
        .commands
        .iter()
        .map(generate_from_parsed_impl_for_opts)
        .collect();

    // Commands enum and match generation
    let match_return = generate_match_return(&dsl);

    let output = quote! {
        // These structs are generated for each command so we can parse into them later
        #(#args_structs)*
        #(#opts_structs)*

        // Generated Commands enum
        enum Command {
            #(#command_fields),*
        }

        let parsed = {
            // FromParsed implementations
            use dsl_cli::dsl_cli_core::FromParsed;
            #(#args_from_parsed)*
            #(#opts_from_parsed)*

            // CLI setup
            #cli_setup

            // Command matching and parsing
            #match_return
        };
    };

    output.into()
}
