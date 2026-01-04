use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

fn impl_from_parsed_args(_derive_input: DeriveInput) -> TokenStream {
    todo!()
}

#[proc_macro_derive(FromParsedArgs)]
pub fn from_parsed_args_derive_macro(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    impl_from_parsed_args(derive_input)
}
