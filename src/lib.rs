#![feature(let_chains)]

use crate::branch::get_mocking_candidate;
use crate::mutations::mutate;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

mod branch;
mod mutations;

#[proc_macro_attribute]
pub fn toffel(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let mut tokens = parse_macro_input!(input as ItemStruct);

    let new_stream = tokens
        .fields
        .iter_mut()
        .find(|field| {
            field
                .attrs
                .iter()
                .any(|attr| attr.meta.path().is_ident("mocked"))
        })
        .map(|field| {
            let mocked_stream = get_mocking_candidate(&field.ty);
            dbg!(&field);
            mutate(field, mocked_stream);
            field
        });

    todo!()
}

/*
#[proc_macro_attribute]
pub fn mock(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(input as ItemStruct);

    todo!()
}
*/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
