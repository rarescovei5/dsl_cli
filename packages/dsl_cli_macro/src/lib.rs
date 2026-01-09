use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Ident, parse_macro_input};

mod utils;
use crate::utils::*;

mod ast;
use crate::ast::*;

mod type_gen;
use crate::type_gen::*;

mod cli_setup_gen;
use crate::cli_setup_gen::*;

// ----------------------------------------------------------------
// Code Generation
// ----------------------------------------------------------------

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
        impl dsl_cli::dsl_cli_core::FromParsed for #struct_name {
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
                impl dsl_cli::dsl_cli_core::FromParsed for #nested_struct_name {
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

        impl dsl_cli::dsl_cli_core::FromParsed for #struct_name {
            fn from_parsed(mut __parsed: ::std::collections::HashMap<String, Box<dyn ::std::any::Any>>) -> Self {
                #(#field_extractions)*
                Self {
                    #(#field_names),*
                }
            }
        }
    }
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
