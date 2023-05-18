#![feature(let_chains)]
#![feature(if_let_guard)]

use crate::extract::{prepare_mock_name, ExtractName, Extracted};
use crate::pimpl::MockContext;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_str, FnArg, ImplItem, Item, ItemImpl, ReturnType, Type, TypePath,
};
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
    let tokens = parse_macro_input!(input as Item);
    let mut mock = tokens.clone();
    match mock {
        Item::Struct(ref mut s) => {
            let name = format!("{}Mock", s.ident);
            s.ident = parse_str(name.as_str()).unwrap();
        }
        Item::Enum(ref mut e) => {
            let name = format!("{}Mock", e.ident);
            e.ident = parse_str(name.as_str()).unwrap();
        }
        _ => todo!(),
    };
    let original_and_mocked_struct = quote! {
        #tokens
        #mock
    };

    TokenStream::from(original_and_mocked_struct)
}

#[allow(clippy::redundant_clone)]
#[proc_macro_attribute]
pub fn mock_impl_and_use_defaults(tokens: TokenStream, input: TokenStream) -> TokenStream {
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
