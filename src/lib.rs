#![feature(let_chains)]

use muast::{get_mocking_candidate, mutate};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};
mod muast;

#[proc_macro_attribute]
pub fn toffel(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let mut tokens = parse_macro_input!(input as ItemStruct);
    dbg!(tokens
        .fields
        .iter()
        .map(|x| &x.ident)
        .collect::<Vec<&Option<syn::Ident>>>());

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
            let mocked_type = get_mocking_candidate(&field.ty);
            mutate(field, mocked_type);
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
