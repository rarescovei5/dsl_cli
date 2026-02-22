use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    Argument, Command, generate_args_struct_name, get_effective_type, is_optional_type, is_variadic_type, parse_flags, to_pascal_case
};

pub fn generate_from_parsed_impl_for_args(cmd: &Command) -> TokenStream2 {
    let struct_name = format_ident!(
        "{}",
        generate_args_struct_name(&to_pascal_case(&cmd.name.to_string()))
    );

    let field_extractions: Vec<TokenStream2> = cmd
        .arguments
        .iter()
        .map(|arg| {
            let field_name = &arg.name;

            let field_name_str = field_name.to_string();

            generate_arg_extraction(field_name, &field_name_str, arg, false)
        })
        .collect();

    let field_names: Vec<&Ident> = cmd.arguments.iter().map(|arg| &arg.name).collect();

    quote! {
        impl dsl_cli::dsl_cli_core::FromParsedArgs for #struct_name {
            fn from_parsed(mut __parsed: dsl_cli::dsl_cli_core::ParsedArgs) -> Self {
                #(#field_extractions)*
                Self {
                    #(#field_names),*
                }
            }
        }
    }
}

pub fn generate_from_parsed_impl_for_opts(cmd: &Command) -> TokenStream2 {
    let struct_name = format_ident!("{}Opts", to_pascal_case(&cmd.name.to_string()));
    let cmd_pascal = to_pascal_case(&cmd.name.to_string());

    let mut nested_impls = Vec::new();
    let mut field_extractions = Vec::new();
    let mut field_names = Vec::new();

    for opt in &cmd.options {
        let (_, _, opt_name) = parse_flags(&opt.flags.value());
        let field_name = format_ident!("{}", opt_name);
        field_names.push(field_name.clone());

        let arg_count = opt.arguments.len();
        let is_optional = !opt.required;

        match (arg_count, is_optional) {
            (0,false) => {
                field_extractions.push(
                    quote! {
                        let #field_name: bool = {
                            let val = __parsed.remove(#opt_name).unwrap();
                            val.as_flag()
                        };
                    }
                );
            }
            (0,true) => {
                field_extractions.push(
                    quote! {
                        let #field_name: bool = {
                            let val = __parsed.remove(#opt_name).unwrap();
                            if !val.is_none() {
                                val.as_flag()
                            } else {
                                false
                            }
                        };
                    }
                );
            }
            (1,is_scope_optional) => {
                let arg = &opt.arguments[0];
                let field_name = &Ident::new(&opt_name, Span::call_site());
                let field_name_str = opt_name;
                field_extractions.push(
                    generate_arg_extraction(field_name, &field_name_str, arg, is_scope_optional)
                );
            }
            (_, is_scope_optional) => {
                // Multiple arguments - use nested struct
                let nested_prefix = format!("{}{}", cmd_pascal, to_pascal_case(&opt_name));
                let nested_struct_name = format_ident!("{}", generate_args_struct_name(&nested_prefix));

                // Generate FromParsed for nested struct
                let nested_field_extractions: Vec<TokenStream2> = opt
                    .arguments
                    .iter()
                    .map(|arg| {
                        let arg_field_name = &arg.name;
                        let arg_field_name_str = arg_field_name.to_string();
                        
                        generate_arg_extraction(arg_field_name, &arg_field_name_str, arg, is_scope_optional)
                    })
                    .collect();

                let nested_field_names: Vec<&Ident> =
                    opt.arguments.iter().map(|arg| &arg.name).collect();

                nested_impls.push(quote! {
                    impl dsl_cli::dsl_cli_core::FromParsedArgs for #nested_struct_name {
                        fn from_parsed(mut __parsed: dsl_cli::dsl_cli_core::ParsedArgs) -> Self {
                            #(#nested_field_extractions)*
                            Self {
                                #(#nested_field_names),*
                            }
                        }
                    }
                });

                if !is_scope_optional {
                    field_extractions.push(quote! {
                        let #field_name: #nested_struct_name = {
                            let val = __parsed.remove(#opt_name).unwrap();
                            #nested_struct_name::from_parsed(val.as_args())
                        };
                    });
                } else {
                    field_extractions.push(quote! {
                        let #field_name: #nested_struct_name = {
                            let val = __parsed.remove(#opt_name).unwrap();
                            if !val.is_none() {
                                #nested_struct_name::from_parsed(val.as_args())
                            } else {
                                #nested_struct_name::default()
                            }
                        };
                    });
                }
            }
        }
    }

    quote! {
        #(#nested_impls)*

        impl dsl_cli::dsl_cli_core::FromParsedOpts for #struct_name {
            fn from_parsed(mut __parsed: dsl_cli::dsl_cli_core::ParsedOpts) -> Self {
                #(#field_extractions)*
                Self {
                    #(#field_names),*
                }
            }
        }
    }
}


fn generate_arg_extraction(
    field_name: &syn::Ident,
    field_name_str: &str,
    arg: &Argument, 
    scope_optional: bool
) -> TokenStream2 {
    let ty = get_effective_type(arg);
    let field_type = if scope_optional {
        syn::parse_quote!(Option<#ty>)
    } else {
        ty
    };
    
    let is_optional = scope_optional || is_optional_type(&arg.ty);
    let is_variadic = is_variadic_type(&arg.ty);
    let has_default = arg.default.is_some();

    match (is_optional,is_variadic, has_default) {
        // T
        (false,false,false) =>{ 
            return quote! {
                let #field_name: #field_type = {
                    let val = __parsed.remove(#field_name_str).unwrap();
                    val.as_value().parse().unwrap()
                };
            };
        }
        // Option<T>
        (true,false,false) => {
            return quote! {
                let #field_name: #field_type = {
                    let val = __parsed.remove(#field_name_str).unwrap();
                    if !val.is_none() {
                        Some(val.as_value().parse().unwrap())
                    } else {
                        None
                    }
                };
            };
        }
        // Vec<T>
        (false,true,false) => {
            return quote! {
                let #field_name: #field_type = {
                    let val = __parsed.remove(#field_name_str).unwrap();
                    let vec_val = val.as_variadic();
                    vec_val.iter().map(|s| s.parse().unwrap()).collect()
                };
            };
        }
        // Option<Vec<T>>
        (true,true,false) => {
            return quote! {
                let #field_name: #field_type = {
                    let val = __parsed.remove(#field_name_str).unwrap();
                    if !val.is_none() {
                        let vec_val = val.as_variadic();
                        Some(vec_val.iter().map(|s| s.parse().unwrap()).collect())
                    } else {
                        None
                    }
                };
            };
        }
        // Option<Vec<T>> with default
        (true,true,true) => {
            let default_val = arg.default.as_ref().unwrap();
            return quote! {
                let #field_name: #field_type = {
                    let val = __parsed.remove(#field_name_str).unwrap();
                    if !val.is_none() {
                        let vec_val = val.as_variadic();
                        vec_val.iter().map(|s| s.parse().unwrap()).collect()
                    } else {
                        #default_val
                    }
                };
            };
        }
        // Option<T> with default
        (true,false,true) => {
            let default_val = arg.default.as_ref().unwrap();
            return quote! {
                let #field_name: #field_type = {
                    let val = __parsed.remove(#field_name_str).unwrap();
                    if !val.is_none() {
                        val.as_value().parse().unwrap()
                    } else {
                        #default_val
                    }
                };
            };
        }
        _ => unreachable!()
    }
}
