use crate::{Argument, CliDsl, CliOption, is_optional_type, is_variadic_type, parse_flags};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn generate_arg_def(arg: &Argument) -> TokenStream2 {
    let arg_name = arg.name.to_string();
    let arg_desc = match &arg.description {
        Some(d) => quote! { Some(#d) },
        None => quote! { None::<&str> },
    };
    let optional = is_optional_type(&arg.ty);
    let variadic = is_variadic_type(&arg.ty);

    quote! {
        dsl_cli::cli_core::CliArgument::new(
            #arg_name.to_string(),
            #arg_desc,
            #optional,
            #variadic,
        )
    }
}

pub fn generate_opt_def(opt: &CliOption) -> TokenStream2 {
    let flags_str = opt.flags.value();
    let (short, long, opt_name) = parse_flags(&flags_str);

    let flags_expr = match (short, long.as_ref()) {
        (Some(s), Some(l)) => {
            quote! { dsl_cli::cli_core::CliOptionFlags::ShortAndLong(#s, #l.to_string()) }
        }
        (Some(s), None) => quote! { dsl_cli::cli_core::CliOptionFlags::Short(#s) },
        (None, Some(l)) => quote! { dsl_cli::cli_core::CliOptionFlags::Long(#l.to_string()) },
        (None, None) => {
            return syn::Error::new(opt.flags.span(), "Invalid option flags").into_compile_error();
        }
    };

    let opt_desc = match &opt.description {
        Some(d) => quote! { Some(#d) },
        None => quote! { None::<&str> },
    };

    let optional = !opt.required;

    let opt_arg_defs: Vec<TokenStream2> = opt.arguments.iter().map(generate_arg_def).collect();

    quote! {
        {
            let mut __opt = dsl_cli::cli_core::CliOption::new(
                #opt_name,
                #flags_expr,
                #opt_desc,
                #optional,
            );

            __opt #(.add_argument(#opt_arg_defs))*;

            __opt
        }
    }
}

pub fn generate_cli_setup(dsl: &CliDsl) -> TokenStream2 {
    let name = &dsl.name;
    let version = &dsl.version;
    let description = &dsl.description;

    let command_registrations: Vec<TokenStream2> = dsl
        .commands
        .iter()
        .map(|cmd| {
            let cmd_name = cmd.name.to_string();
            let cmd_desc = match &cmd.description {
                Some(d) => quote! { Some(#d) },
                None => quote! { None::<&str> },
            };

            let arg_defs: Vec<TokenStream2> = cmd.arguments.iter().map(generate_arg_def).collect();

            let opt_registrations: Vec<TokenStream2> =
                cmd.options.iter().map(generate_opt_def).collect();

            quote! {
                {
                    let __cmd = __cli.add_command(#cmd_name, #cmd_desc);
                    __cmd
                        #(.add_argument(#arg_defs))*
                        #(.add_option(#opt_registrations))*;
                }
            }
        })
        .collect();

    quote! {
        let mut __cli = dsl_cli::cli_core::Cli::new(#name, #version, #description);
        #(#command_registrations)*
    }
}
