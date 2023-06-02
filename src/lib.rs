#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(exact_size_is_empty)]
#![feature(extend_one)]
#![feature(drain_filter)]
#![feature(drain_keep_rest)]
#![allow(unused)]

use crate::pimpl::MockContext;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, Item, ItemImpl};
use toffel::Toffelise;

mod branch;
mod extract;
mod pimpl;
mod toffel;

#[proc_macro_attribute]
pub fn toffel(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(input as Item);

    match tokens {
        Item::Struct(s) => s.replace_mocks(),
        Item::Enum(e) => e.replace_mocks(),
        _ => todo!(),
    }
}

#[proc_macro_attribute]
pub fn mock(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let mut tokens = parse_macro_input!(input as Item);
    let mut mock = tokens.clone();
    let fields = match mock {
        Item::Struct(ref mut s) => {
            let name = format!("{}Mock", s.ident);
            s.ident = parse_str(name.as_str()).unwrap();
            let f = extract::parse_fields_and_generate_for_values(s);
            extract::clean_out_attributes(&mut tokens);
            f
        }
        Item::Enum(ref mut e) => {
            let name = format!("{}Mock", e.ident);
            e.ident = parse_str(name.as_str()).unwrap();
            let f = extract::parse_fields_and_generate_variant(e);
            extract::clean_out_attributes(&mut tokens);
            f
        }
        _ => todo!(),
    };

    extract::clean_out_attributes(&mut mock);
    let mut original_and_mocked_struct = TokenStream::from(quote! {
        #tokens
        #mock
    });

    original_and_mocked_struct.extend_one(fields);
    original_and_mocked_struct
}

#[allow(clippy::redundant_clone)]
#[proc_macro_attribute]
pub fn mock_impl(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(input as ItemImpl);
    let original_type = tokens.self_ty.clone();
    let context = MockContext::new(original_type);
    context.mock_impl(tokens)
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
