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

fn tokens_for_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let ident = field.ident.clone().unwrap();
    let name = ident.to_string();
    let ty = &field.ty;

    if let Some(inner) = get_option_inner(ty) {
        quote! {
            #ident: parsed_args
                .remove(#name)
                .expect("Key always exists")
                .map(|any| *any.downcast::<#inner>().unwrap())
        }
    } else {
        quote! {
            #ident: *parsed_args
                .remove(#name)
                .expect("Key always exists")
                .unwrap()
                .downcast::<#ty>()
                .unwrap()
        }
    }
    .into()
}

fn impl_from_parsed_args(derive_input: DeriveInput) -> TokenStream {
    let struct_name = derive_input.ident.clone();

    let data = match &derive_input.data {
        syn::Data::Struct(data) => data,
        _ => panic!("FromParsedArgs can only be derived for structs"),
    };

    let fields = data.fields.clone();

    let tokens = fields.iter().map(tokens_for_field).collect::<Vec<_>>();

    quote! {
        impl FromParsedArgs for #struct_name {
            fn from_parsed_args(mut parsed_args: ParsedArgs) -> Self {
                #struct_name {
                    #(#tokens),*
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(FromParsedArgs)]
pub fn from_parsed_args_derive_macro(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    impl_from_parsed_args(derive_input)
}
