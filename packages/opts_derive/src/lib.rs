use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, GenericArgument, PathArguments, Type, TypePath, parse_macro_input};

fn get_option_inner(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

fn is_bool(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "bool";
        }
    }
    false
}
fn tokens_for_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let ident = field.ident.clone().unwrap();
    let name = ident.to_string();
    let ty = &field.ty;

    let inner_opt = get_option_inner(ty);
    let is_ty_bool = is_bool(ty);
    let is_inner_bool = inner_opt.map(|t| is_bool(t)).unwrap_or(false);

    if is_ty_bool {
        quote! {
            #ident: parsed_opts
                .remove(#name)
                .expect("Key always exists")
                .unwrap()
                .into_flag()
        }
    } else if is_inner_bool {
        quote! {
            #ident: parsed_opts
                .remove(#name)
                .expect("Key always exists")
                .map(|v| v.into_flag())
        }
    } else if let Some(inner) = inner_opt {
        quote! {
            #ident: parsed_opts
                .remove(#name)
                .expect("Key always exists")
                .map(|v| v.into_args())
                .map(|args| <#inner>::from_parsed_args(args))
        }
    } else {
        quote! {
            #ident: <#ty>::from_parsed_args(
                parsed_opts
                    .remove(#name)
                    .expect("Key always exists")
                    .unwrap()
                    .into_args()
            )
        }
    }
    .into()
}

fn impl_from_parsed_opts(derive_input: DeriveInput) -> TokenStream {
    let struct_name = derive_input.ident.clone();

    let data = match &derive_input.data {
        syn::Data::Struct(data) => data,
        _ => panic!("FromParsedOpts can only be derived for structs"),
    };

    let fields = data.fields.clone();
    let tokens = fields.iter().map(tokens_for_field).collect::<Vec<_>>();

    quote! {
        impl FromParsedOpts for #struct_name {
            fn from_parsed_opts(mut parsed_opts: ParsedOpts) -> Self {
                #struct_name {
                    #(#tokens),*
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(FromParsedOpts)]
pub fn from_parsed_opts_derive_macro(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    impl_from_parsed_opts(derive_input)
}
