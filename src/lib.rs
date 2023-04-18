#![feature(let_chains)]

use crate::branch::get_mocking_candidate;
use crate::extract::MockableType;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, Fields, ItemStruct};

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
    let tokens = parse_macro_input!(input as MockableType);
    let mut mock = tokens.clone();
    match mock {
        MockableType::Struct(ref mut s) => {
            let name = format!("{}Mock", s.ident);
            s.ident = parse_str(name.as_str()).unwrap();
        }
        MockableType::Enum(ref mut e) => {
            let name = format!("{}Mock", e.ident);
            e.ident = parse_str(name.as_str()).unwrap();
        }
    };
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
