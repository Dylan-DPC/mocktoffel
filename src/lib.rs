#![feature(let_chains)]

use crate::branch::get_mocking_candidate;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, ItemStruct, parse_str};

mod branch;
mod extract;

#[proc_macro_attribute]
pub fn toffel(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(input as ItemStruct);

    let fields: Vec<syn::Field> = match tokens.fields {
        Fields::Named(named) => named
            .named
            .into_iter()
            .map(|mut field| {
                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.meta.path().is_ident("mocked"))
                {
                    field.attrs = vec![];
                    field.ty = get_mocking_candidate(&field.ty).mocked_type;
                    field
                } else {
                    field
                }
            })
            .collect(),
        _ => todo!(),
    };

    let struct_name = tokens.ident;
    let generics = tokens.generics;

    let struct_with_mocks_added = quote! {
           struct #struct_name #generics {
                 #(#fields),*
           }

    };

    TokenStream::from(struct_with_mocks_added)
}

#[proc_macro_attribute]
pub fn mock(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(input as ItemStruct);
    let mut mock = tokens.clone();
    let name = mock.ident;
    let mock_name = format!("{name}Mock");
    mock.ident = parse_str(mock_name.as_str()).unwrap();
    let original_and_mocked_struct = quote! {
        #tokens
        #mock
    };

    TokenStream::from(original_and_mocked_struct)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
