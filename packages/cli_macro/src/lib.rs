use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Ident, parse_macro_input};

mod utils;
use crate::utils::*;

mod ast;
use crate::ast::*;

// ----------------------------------------------------------------
// Code Generation
// ----------------------------------------------------------------

fn generate_cli_setup(dsl: &CliDsl) -> TokenStream2 {
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

            let arg_registrations: Vec<TokenStream2> = cmd
                .arguments
                .iter()
                .map(|arg| {
                    let arg_name = arg.name.to_string();
                    let arg_desc = match &arg.description {
                        Some(d) => quote! { Some(#d) },
                        None => quote! { None::<&str> },
                    };
                    let optional = is_optional_type(&arg.ty);
                    let variadic = is_variadic_type(&arg.ty);

                    quote! {
                        __cmd.add_argument(::core_lib::CliArgument::new(
                            #arg_name.to_string(),
                            #arg_desc,
                            #optional,
                            #variadic,
                        ));
                    }
                })
                .collect();

            let opt_registrations: Vec<TokenStream2> = cmd
                .options
                .iter()
                .map(|opt| {
                    let flags_str = opt.flags.value();
                    let (short, long, opt_name) = parse_flags(&flags_str);

                    let flags_expr = match (short, long.as_ref()) {
                        (Some(s), Some(l)) => {
                            quote! { ::common::CliOptionFlags::ShortAndLong(#s, #l.to_string()) }
                        }
                        (Some(s), None) => quote! { ::common::CliOptionFlags::Short(#s) },
                        (None, Some(l)) => {
                            quote! { ::common::CliOptionFlags::Long(#l.to_string()) }
                        }
                        (None, None) => quote! { ::common::CliOptionFlags::Long("".to_string()) },
                    };

                    let opt_desc = match &opt.description {
                        Some(d) => quote! { Some(#d) },
                        None => quote! { None::<&str> },
                    };

                    let optional = !opt.required;

                    let opt_arg_registrations: Vec<TokenStream2> = opt
                        .arguments
                        .iter()
                        .map(|arg| {
                            let arg_name = arg.name.to_string();
                            let arg_desc = match &arg.description {
                                Some(d) => quote! { Some(#d) },
                                None => quote! { None::<&str> },
                            };
                            let arg_optional = is_optional_type(&arg.ty);
                            let arg_variadic = is_variadic_type(&arg.ty);

                            quote! {
                                __opt.add_argument(::core_lib::CliArgument::new(
                                    #arg_name.to_string(),
                                    #arg_desc,
                                    #arg_optional,
                                    #arg_variadic,
                                ));
                            }
                        })
                        .collect();

                    quote! {
                        {
                            let mut __opt = ::core_lib::CliOption::new(
                                #opt_name,
                                #flags_expr,
                                #opt_desc,
                                #optional,
                            );
                            #(#opt_arg_registrations)*
                            __cmd.add_option(__opt);
                        }
                    }
                })
                .collect();

            quote! {
                {
                    let __cmd = __cli.add_command(#cmd_name, #cmd_desc);
                    #(#arg_registrations)*
                    #(#opt_registrations)*
                }
            }
        })
        .collect();

    quote! {
        let mut __cli = ::core_lib::Cli::new(#name, #version, #description);
        #(#command_registrations)*
    }
}

fn generate_args_struct(cmd: &Command) -> TokenStream2 {
    let struct_name = format_ident!("{}Args", to_pascal_case(&cmd.name.to_string()));

    let fields: Vec<TokenStream2> = cmd
        .arguments
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

fn generate_opts_struct(cmd: &Command) -> TokenStream2 {
    let struct_name = format_ident!("{}Opts", to_pascal_case(&cmd.name.to_string()));
    let cmd_pascal = to_pascal_case(&cmd.name.to_string());

    let mut nested_structs = Vec::new();
    let mut fields = Vec::new();

    for opt in &cmd.options {
        let (_, _, opt_name) = parse_flags(&opt.flags.value());
        let field_name = format_ident!("{}", opt_name);

        if opt.arguments.is_empty() {
            // Boolean flag
            fields.push(quote! { pub #field_name: bool });
        } else if opt.arguments.len() == 1 {
            // Single argument - use the arg's type directly
            let arg = &opt.arguments[0];
            let field_type = get_effective_type(arg);
            fields.push(quote! { pub #field_name: #field_type });
        } else {
            // Multiple arguments - create nested struct
            let nested_struct_name =
                format_ident!("{}{}Opt", cmd_pascal, to_pascal_case(&opt_name));

            let nested_fields: Vec<TokenStream2> = opt
                .arguments
                .iter()
                .map(|arg| {
                    let arg_field_name = &arg.name;
                    let arg_field_type = get_effective_type(arg);
                    quote! { pub #arg_field_name: #arg_field_type }
                })
                .collect();

            nested_structs.push(quote! {
                #[derive(Debug)]
                struct #nested_struct_name {
                    #(#nested_fields),*
                }
            });

            fields.push(quote! { pub #field_name: #nested_struct_name });
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

fn generate_from_parsed_impl_for_args(cmd: &Command) -> TokenStream2 {
    let struct_name = format_ident!("{}Args", to_pascal_case(&cmd.name.to_string()));

    let field_extractions: Vec<TokenStream2> = cmd
        .arguments
        .iter()
        .map(|arg| {
            let field_name = &arg.name;
            let field_name_str = field_name.to_string();
            let field_type = get_effective_type(arg);
            let is_optional = is_optional_type(&arg.ty);
            let is_variadic = is_variadic_type(&arg.ty);
            let has_default = arg.default.is_some();

            if is_variadic && !is_optional {
                // Vec<T>
                quote! {
                    let #field_name: #field_type = {
                        let val = __parsed.remove(#field_name_str).unwrap();
                        if let Some(vec_val) = val.downcast_ref::<Vec<String>>() {
                            vec_val.iter().map(|s| s.parse().unwrap()).collect()
                        } else if let Some(str_val) = val.downcast_ref::<String>() {
                            vec![str_val.parse().unwrap()]
                        } else {
                            Vec::new()
                        }
                    };
                }
            } else if is_optional && !has_default {
                // Option<T>
                if is_variadic_type(&field_type) {
                    // Option<Vec<T>>
                    quote! {
                        let #field_name: #field_type = {
                            let val = __parsed.remove(#field_name_str).unwrap();
                            if let Some(vec_val) = val.downcast_ref::<Vec<String>>() {
                                Some(vec_val.iter().map(|s| s.parse().unwrap()).collect())
                            } else if let Some(str_val) = val.downcast_ref::<String>() {
                                Some(vec![str_val.parse().unwrap()])
                            } else {
                                None
                            }
                        };
                    }
                } else {
                    // Option<T>
                    quote! {
                        let #field_name: #field_type = {
                            let val = __parsed.remove(#field_name_str).unwrap();
                            if let Some(str_val) = val.downcast_ref::<String>() {
                                Some(str_val.parse().unwrap())
                            } else {
                                None
                            }
                        };
                    }
                }
            } else if has_default {
                // Has default - use the default value if None
                let default_val = arg.default.as_ref().unwrap();
                quote! {
                    let #field_name: #field_type = {
                        let val = __parsed.remove(#field_name_str).unwrap();
                        if let Some(str_val) = val.downcast_ref::<String>() {
                            str_val.parse().unwrap()
                        } else {
                            let default_str = stringify!(#default_val);
                            let cleaned = default_str.trim_matches('"');
                            cleaned.parse().unwrap()
                        }
                    };
                }
            } else {
                // Regular required type T
                quote! {
                    let #field_name: #field_type = {
                        let val = __parsed.remove(#field_name_str).unwrap();
                        val.downcast_ref::<String>().unwrap().parse().unwrap()
                    };
                }
            }
        })
        .collect();

    let field_names: Vec<&Ident> = cmd.arguments.iter().map(|arg| &arg.name).collect();

    quote! {
        impl ::core_lib::FromParsed for #struct_name {
            fn from_parsed(mut __parsed: ::std::collections::HashMap<String, Box<dyn ::std::any::Any>>) -> Self {
                #(#field_extractions)*
                Self {
                    #(#field_names),*
                }
            }
        }
    }
}

fn generate_from_parsed_impl_for_opts(cmd: &Command) -> TokenStream2 {
    let struct_name = format_ident!("{}Opts", to_pascal_case(&cmd.name.to_string()));
    let cmd_pascal = to_pascal_case(&cmd.name.to_string());

    let mut nested_impls = Vec::new();
    let mut field_extractions = Vec::new();
    let mut field_names = Vec::new();

    for opt in &cmd.options {
        let (_, _, opt_name) = parse_flags(&opt.flags.value());
        let field_name = format_ident!("{}", opt_name);
        field_names.push(field_name.clone());

        if opt.arguments.is_empty() {
            // Boolean flag
            field_extractions.push(quote! {
                let #field_name: bool = {
                    let val = __parsed.remove(#opt_name).unwrap();
                    if let Some(str_val) = val.downcast_ref::<String>() {
                        str_val == "true"
                    } else {
                        false
                    }
                };
            });
        } else if opt.arguments.len() == 1 {
            // Single argument
            let arg = &opt.arguments[0];
            let field_type = get_effective_type(arg);
            let is_optional = is_optional_type(&arg.ty);
            let has_default = arg.default.is_some();

            if is_optional && !has_default {
                field_extractions.push(quote! {
                    let #field_name: #field_type = {
                        let val = __parsed.remove(#opt_name).unwrap();
                        if let Some(str_val) = val.downcast_ref::<String>() {
                            Some(str_val.parse().unwrap())
                        } else {
                            None
                        }
                    };
                });
            } else if has_default {
                let default_val = arg.default.as_ref().unwrap();
                field_extractions.push(quote! {
                    let #field_name: #field_type = {
                        let val = __parsed.remove(#opt_name).unwrap();
                        if let Some(str_val) = val.downcast_ref::<String>() {
                            str_val.parse().unwrap()
                        } else {
                            let default_str = stringify!(#default_val);
                            let cleaned = default_str.trim_matches('"');
                            cleaned.parse().unwrap()
                        }
                    };
                });
            } else {
                field_extractions.push(quote! {
                    let #field_name: #field_type = {
                        let val = __parsed.remove(#opt_name).unwrap();
                        val.downcast_ref::<String>().unwrap().parse().unwrap()
                    };
                });
            }
        } else {
            // Multiple arguments - use nested struct
            let nested_struct_name =
                format_ident!("{}{}Opt", cmd_pascal, to_pascal_case(&opt_name));

            // Generate FromParsed for nested struct
            let nested_field_extractions: Vec<TokenStream2> = opt
                .arguments
                .iter()
                .map(|arg| {
                    let arg_field_name = &arg.name;
                    let arg_field_name_str = arg_field_name.to_string();
                    let arg_field_type = get_effective_type(arg);
                    let is_optional = is_optional_type(&arg.ty);
                    let has_default = arg.default.is_some();

                    if is_optional && !has_default {
                        quote! {
                            let #arg_field_name: #arg_field_type = {
                                let val = __parsed.remove(#arg_field_name_str).unwrap();
                                if let Some(str_val) = val.downcast_ref::<String>() {
                                    Some(str_val.parse().unwrap())
                                } else {
                                    None
                                }
                            };
                        }
                    } else if has_default {
                        let default_val = arg.default.as_ref().unwrap();
                        quote! {
                            let #arg_field_name: #arg_field_type = {
                                let val = __parsed.remove(#arg_field_name_str).unwrap();
                                if let Some(str_val) = val.downcast_ref::<String>() {
                                    str_val.parse().unwrap()
                                } else {
                                    let default_str = stringify!(#default_val);
                                    let cleaned = default_str.trim_matches('"');
                                    cleaned.parse().unwrap()
                                }
                            };
                        }
                    } else {
                        quote! {
                            let #arg_field_name: #arg_field_type = {
                                let val = __parsed.remove(#arg_field_name_str).unwrap();
                                val.downcast_ref::<String>().unwrap().parse().unwrap()
                            };
                        }
                    }
                })
                .collect();

            let nested_field_names: Vec<&Ident> =
                opt.arguments.iter().map(|arg| &arg.name).collect();

            nested_impls.push(quote! {
                impl ::core_lib::FromParsed for #nested_struct_name {
                    fn from_parsed(mut __parsed: ::std::collections::HashMap<String, Box<dyn ::std::any::Any>>) -> Self {
                        #(#nested_field_extractions)*
                        Self {
                            #(#nested_field_names),*
                        }
                    }
                }
            });

            field_extractions.push(quote! {
                let #field_name: #nested_struct_name = {
                    let val = __parsed.remove(#opt_name).unwrap();
                    if let Some(inner_map) = val.downcast::<::std::collections::HashMap<String, Box<dyn ::std::any::Any>>>().ok() {
                        #nested_struct_name::from_parsed(*inner_map)
                    } else {
                        panic!("Expected nested option arguments for {}", #opt_name)
                    }
                };
            });
        }
    }

    quote! {
        #(#nested_impls)*

        impl ::core_lib::FromParsed for #struct_name {
            fn from_parsed(mut __parsed: ::std::collections::HashMap<String, Box<dyn ::std::any::Any>>) -> Self {
                #(#field_extractions)*
                Self {
                    #(#field_names),*
                }
            }
        }
    }
}

fn generate_enum_match(dsl: &CliDsl) -> (TokenStream2, TokenStream2) {
    let enum_members: Vec<TokenStream2> = dsl
        .commands
        .iter()
        .map(|cmd| {
            let cmd_name_str = format_ident!("{}", to_pascal_case(&cmd.name.to_string()));
            let args_struct = format_ident!("{}Args", to_pascal_case(&cmd.name.to_string()));
            let opts_struct = format_ident!("{}Opts", to_pascal_case(&cmd.name.to_string()));
            quote! { #cmd_name_str(#args_struct, #opts_struct) }
        })
        .collect();

    let match_arms: Vec<TokenStream2> = dsl
        .commands
        .iter()
        .filter_map(|cmd| {
            let cmd_name_str = cmd.name.to_string();

            if cmd_name_str == "cli" {
                return None;
            }

            let cmd_name_pascal = format_ident!("{}", to_pascal_case(&cmd.name.to_string()));

            let args_struct = format_ident!("{}Args", to_pascal_case(&cmd.name.to_string()));
            let opts_struct = format_ident!("{}Opts", to_pascal_case(&cmd.name.to_string()));

            // Commands without actions print debug info about parsed args/opts
            Some(quote! {
                #cmd_name_str => {
                    let (__args, __opts) = __cli.parse::<#args_struct, #opts_struct>(
                        __command_name.to_string(),
                        __args_to_parse,
                    ).unwrap();
                    Command::#cmd_name_pascal(__args, __opts)
                }
            })
        })
        .collect();

    let global_arm = match dsl.commands.iter().find(|cmd| cmd.name == "cli") {
        Some(_) => quote! {
            _ => {
                let (__args, __opts) = __cli.parse::<::CliArgs, ::CliOpts>(
                    "cli".to_string(),
                    __args_to_parse,
                ).unwrap();

                Command::Cli(__args, __opts)
            }
        },
        None => quote! {
            _ => {
                std::process::exit(1)
            }
        },
    };

    (
        quote! {
            enum Command {
                #(#enum_members),*
            }
        },
        quote! {
                let __env_args: Vec<String> = ::std::env::args().collect();
                let __command_name = __env_args.get(1).map(|s| s.as_str()).unwrap_or("");
                let __args_to_parse: Vec<String> = __env_args.get(2..).map(|s| s.to_vec()).unwrap_or_default();

                match __command_name {
                    #(#match_arms)*
                    #global_arm
                }
        },
    )
}

// ----------------------------------------------------------------
// Main Macro
// ----------------------------------------------------------------

#[proc_macro]
pub fn cli(input: TokenStream) -> TokenStream {
    let dsl = parse_macro_input!(input as CliDsl);

    // Generate CLI setup
    let cli_setup = generate_cli_setup(&dsl);

    // Generate structs for each command
    let args_structs: Vec<TokenStream2> = dsl.commands.iter().map(generate_args_struct).collect();

    let opts_structs: Vec<TokenStream2> = dsl.commands.iter().map(generate_opts_struct).collect();

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
    let (enum_commands_def, match_run) = generate_enum_match(&dsl);

    let output = quote! {
        // Generated Args structs
        #(#args_structs)*

        // Generated Opts structs
        #(#opts_structs)*

        // Generated Commands enum
        #enum_commands_def

        let parsed = {
            use ::core_lib::FromParsed;

            // FromParsed implementations for Args
            #(#args_from_parsed)*

            // FromParsed implementations for Opts
            #(#opts_from_parsed)*

            // CLI setup
            #cli_setup

            // Command matching and parsing
            #match_run
        };
    };

    output.into()
}
