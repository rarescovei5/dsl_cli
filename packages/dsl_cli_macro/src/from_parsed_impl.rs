use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    Command, generate_args_struct_name, get_effective_type, is_optional_type, is_variadic_type,
    parse_flags, to_pascal_case,
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

        if opt.arguments.is_empty() {
            // Boolean flag
            field_extractions.push(quote! {
                let #field_name: bool = {
                    let val = __parsed.remove(#opt_name).unwrap();
                    if let Some(bool_val) = val.downcast_ref::<bool>() {
                        *bool_val
                    } else if let Some(str_val) = val.downcast_ref::<String>() {
                        str_val == "true"
                    } else {
                        false
                    }
                };
            });
        } else if opt.arguments.len() == 1 {
            // Single argument
            let arg = &opt.arguments[0];
            let is_optional = is_optional_type(&arg.ty);
            let is_variadic = is_variadic_type(&arg.ty);
            let has_default = arg.default.is_some();

            // Field type selection:
            // - Defaults always yield a concrete value: unwrap Option<T> when a default exists.
            // - Optional options yield Option<T> unless the arg is already Option<T>.
            let field_type = if has_default {
                get_effective_type(arg)
            } else if opt.required {
                arg.ty.clone()
            } else if is_optional {
                arg.ty.clone()
            } else {
                let ty = arg.ty.clone();
                syn::parse_quote!(Option<#ty>)
            };

            // If the option itself is optional, the field should be Option<...> (unless default).
            let output_is_option = !has_default && (!opt.required || is_optional);

            if output_is_option {
                if is_variadic {
                    field_extractions.push(quote! {
                        let #field_name: #field_type = {
                            let val = __parsed.remove(#opt_name).unwrap();
                            if let Some(vec_val) = val.downcast_ref::<Vec<String>>() {
                                Some(vec_val.iter().map(|s| s.parse().unwrap()).collect())
                            } else if let Some(str_val) = val.downcast_ref::<String>() {
                                Some(vec![str_val.parse().unwrap()])
                            } else {
                                None
                            }
                        };
                    });
                } else {
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
                }
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
            } else if is_variadic {
                field_extractions.push(quote! {
                    let #field_name: #field_type = {
                        let val = __parsed.remove(#opt_name).unwrap();
                        if let Some(vec_val) = val.downcast_ref::<Vec<String>>() {
                            vec_val.iter().map(|s| s.parse().unwrap()).collect()
                        } else if let Some(str_val) = val.downcast_ref::<String>() {
                            vec![str_val.parse().unwrap()]
                        } else {
                            panic!("Missing value for required option '{}'", #opt_name);
                        }
                    };
                });
            } else {
                field_extractions.push(quote! {
                    let #field_name: #field_type = {
                        let val = __parsed.remove(#opt_name).unwrap();
                        val.downcast_ref::<String>()
                            .unwrap_or_else(|| panic!("Missing value for required option '{}'", #opt_name))
                            .parse()
                            .unwrap()
                    };
                });
            }
        } else {
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
                    let is_optional = is_optional_type(&arg.ty);
                    let is_variadic = is_variadic_type(&arg.ty);
                    let has_default = arg.default.is_some();

           
                    let arg_field_type = if has_default {
                        get_effective_type(arg)
                    } else if opt.required {
                        arg.ty.clone()
                    } else if is_optional {
                        arg.ty.clone()
                    } else {
                        let ty = arg.ty.clone();
                        syn::parse_quote!(Option<#ty>)
                    };

                    // When the resulting field type is Option<...>, decode as optional.
                    let output_is_option = !has_default && (!opt.required || is_optional);

                    if has_default {
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
                    } else if output_is_option {
                        // Option<T> / Option<Vec<T>>
                        if is_variadic {
                            quote! {
                                let #arg_field_name: #arg_field_type = {
                                    let val = __parsed.remove(#arg_field_name_str).unwrap();
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
                        }
                    } else if is_variadic {
                        // Vec<T>
                        quote! {
                            let #arg_field_name: #arg_field_type = {
                                let val = __parsed.remove(#arg_field_name_str).unwrap();
                                if let Some(vec_val) = val.downcast_ref::<Vec<String>>() {
                                    vec_val.iter().map(|s| s.parse().unwrap()).collect()
                                } else if let Some(str_val) = val.downcast_ref::<String>() {
                                    vec![str_val.parse().unwrap()]
                                } else {
                                    panic!("Missing value for required option argument '{}'", #arg_field_name_str);
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
