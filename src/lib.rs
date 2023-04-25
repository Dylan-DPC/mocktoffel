#![feature(let_chains)]

use crate::branch::get_mocking_candidate;
use crate::extract::{prepare_mock_name, ExtractName, MockableType};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_str, Fields, ImplItem, ItemImpl, ItemStruct, ReturnType, Type,
    TypePath,
};

#[deny(clippy::pedantic)]
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

#[proc_macro_attribute]
pub fn mock_impl_and_use_defaults(tokens: TokenStream, input: TokenStream) -> TokenStream {
    let tokens = parse_macro_input!(input as ItemImpl);
    let original_name = tokens.self_ty.extract_name();
    let name = prepare_mock_name(&original_name);
    let items = tokens.items.clone();
    let functions = items.into_iter().map(|it| {
        match it {
            ImplItem::Const(c) => todo!(),
            ImplItem::Fn(f) => {


        let visibility = f.vis;
        let function_name = f.sig.ident;
        let inputs = f.sig.inputs.iter();

        match f.sig.output {
            ReturnType::Type(_, p) if matches!(&*p, Type::Path(TypePath {path: pat, .. }) if pat.extract_name() == original_name) => {
                quote! {
                    #visibility fn #function_name(#(#inputs), *) -> #name {
                        Default::default()
                    }
                }
            },
            ReturnType::Type(_, ty) => {
                quote! {
                    #visibility fn #function_name(#(#inputs),*) -> #ty {
                        Default::default()
                    }
                }
            },
            _ => {
                quote! {
                    #visibility fn #function_name(#(#inputs),*) {
                    }
                }
            }
        }

            },
            _ => todo!(),
        }
    });

    TokenStream::from(quote! {
        #tokens
        impl #name {
        #(#functions)*
        }
    })
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
