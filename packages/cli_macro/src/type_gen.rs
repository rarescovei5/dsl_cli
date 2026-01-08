use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{
    Argument, CliDsl, CliOption, Command, generate_args_struct_name, generate_opts_struct_name,
    get_effective_type, parse_flags, to_pascal_case,
};

pub fn generate_args_struct(args: &Vec<Argument>, pascal_prefix: &str) -> TokenStream2 {
    let struct_name = format_ident!("{}", generate_args_struct_name(pascal_prefix));

    let fields: Vec<TokenStream2> = args
        .iter()
        .map(|arg| {
            let field_name = &arg.name;
            let field_type = get_effective_type(arg);
            quote! { pub #field_name: #field_type }
        })
        .collect();

    quote! {
        #[derive(Debug)]
        struct #struct_name {
            #(#fields),*
        }
    }
}

pub fn generate_opts_struct(opts: &Vec<CliOption>, pascal_prefix: &str) -> TokenStream2 {
    let struct_name = format_ident!("{}", generate_opts_struct_name(pascal_prefix));

    let mut nested_structs = Vec::new();
    let mut fields = Vec::new();

    for opt in opts {
        let (_, _, opt_name) = parse_flags(&opt.flags.value());
        let field_name = format_ident!("{}", opt_name);

        match opt.arguments.len() {
            0 => fields.push(quote! { pub #field_name: bool }),
            1 => {
                let arg = &opt.arguments[0];
                let field_type = get_effective_type(arg);
                fields.push(quote! { pub #field_name: #field_type });
            }
            _ => {
                let pascal_prefix = format!("{}{}", pascal_prefix, to_pascal_case(&opt_name));
                let nested_struct = generate_args_struct(&opt.arguments, &pascal_prefix);

                nested_structs.push(nested_struct);

                let field_type = generate_args_struct_name(&pascal_prefix);

                fields.push(quote! { pub #field_name: #field_type });
            }
        }
    }

    quote! {
        #(#nested_structs)*

        #[derive(Debug)]
        struct #struct_name {
            #(#fields),*
        }
    }
}

// Generate the fields for the command enum
pub fn generate_fields_for_command_enum(commands: &Vec<Command>) -> Vec<TokenStream2> {
    let mut fields = Vec::new();
    for cmd in commands {
        let cmd_name_pascal = to_pascal_case(&cmd.name.to_string());
        let cmd_name_str = format_ident!("{}", cmd_name_pascal);
        let args_struct = format_ident!("{}", generate_args_struct_name(&cmd_name_pascal));
        let opts_struct = format_ident!("{}", generate_opts_struct_name(&cmd_name_pascal));
        fields.push(quote! { #cmd_name_str(#args_struct, #opts_struct) });
    }
    fields
}

pub fn generate_match_return(dsl: &CliDsl) -> TokenStream2 {
    let mut match_arms: Vec<TokenStream2> = dsl
        .commands
        .iter()
        .filter_map(|cmd| {
            let cmd_name_str = cmd.name.to_string();

            // We need to add the cli command last since it is a "_" pattern which will match for everything
            // including our existing commands
            if cmd_name_str == "cli" {
                return None;
            }

            let cmd_name_pascal = to_pascal_case(&cmd_name_str);

            let cmd_ident = format_ident!("{}", cmd_name_pascal);

            let args_struct = format_ident!("{}", generate_args_struct_name(&cmd_name_pascal));
            let opts_struct = format_ident!("{}", generate_opts_struct_name(&cmd_name_pascal));

            // Commands without actions print debug info about parsed args/opts
            Some(quote! {
                #cmd_name_str => {
                    Command::#cmd_ident(
                        #args_struct::from_parsed(__parsed_args),
                        #opts_struct::from_parsed(__parsed_opts)
                    )
                }
            })
        })
        .collect();

    if dsl.commands.iter().any(|cmd| cmd.name.to_string() == "cli") {
        match_arms.push(quote! {
            _ => {
                Command::Cli(
                    CliArgs::from_parsed(__parsed_args),
                    CliOpts::from_parsed(__parsed_opts)
                )
            }
        });
    }

    quote! {
        let __env_args: Vec<String> = ::std::env::args().skip(1).collect();
        let __command_name = __env_args.first().map(|s| s.as_str()).unwrap_or("").to_string();

        let (__parsed_args, __parsed_opts) = __cli.parse(__command_name.clone(), __env_args);

        match __command_name.as_str() {
            #(#match_arms),*
            _ => unreachable!()
        }
    }
}
