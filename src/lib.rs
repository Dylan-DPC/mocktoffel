#![feature(let_chains)]

use crate::extract::{prepare_mock_name, ExtractName};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_str, FnArg, ImplItem, Item, ItemImpl, ReturnType, Type, TypePath,
};
use toffel::Toffelise;

mod branch;
mod extract;
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
    let impl_generics = tokens.generics;
    let original_name = original_type.extract_name();
    let mocked_extract = prepare_mock_name(&original_name);
    let generics = original_name.clone().generics;
    let name = mocked_extract.name;
    let items = tokens.items.clone();

    let functions = items.into_iter().map(|it| {
        match it {
            ImplItem::Const(c) => todo!(),
            ImplItem::Fn(mut f) => {
        let visibility = f.vis;
        let function_name = f.sig.ident;
        f.sig.inputs.iter_mut().filter(|arg| {
            matches!(arg, FnArg::Typed(typ) if typ.ty == original_type)
        }).for_each(|arg| {
            if let FnArg::Typed(typ) = arg {
                typ.ty = Box::new(Type::Path(parse_str(format!("{}", name).as_str()).unwrap()));

            }
            else {
                unreachable!()
            }
        });

        let inputs = f.sig.inputs.iter();


        match f.sig.output {
            ReturnType::Type(_, p)
                if matches!(&*p, Type::Path(TypePath {path: pat, .. }) if pat.extract_name() == original_name) => {
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
            ReturnType::Default => {
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

    if let Some((_, tr, _)) = tokens.trait_ {
        let trait_ = tr.extract_name().name;
        TokenStream::from(quote! {
        impl #trait_ for #name {
        #(#functions)*
        }
            })
    } else {
        TokenStream::from(quote! {
            impl #impl_generics #name #generics {
           #(#functions)*
           }
        })
    }
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
