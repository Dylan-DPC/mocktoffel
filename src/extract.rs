use crate::branch::Traitified;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream, Result},
    Ident, ItemEnum, ItemStruct, Path, Token, TraitBoundModifier, Type, TypeParamBound, TypePath,
};

pub struct MockPrepared {
    pub mocked_type: Type,
    stream: Option<TokenStream>,
}

impl MockPrepared {
    pub fn new(mocked_type: Type, stream: Option<TokenStream>) -> Self {
        Self {
            mocked_type,
            stream,
        }
    }
}

pub trait ExtractName {
    fn extract_name(&self) -> Ident;
}

impl ExtractName for Type {
    fn extract_name(&self) -> Ident {
        match self {
            Type::Array(_) => todo!(),
            Type::BareFn(f) => todo!(),
            Type::Group(g) => todo!(),
            Type::ImplTrait(im) => extract_name_for_bounds(im),
            Type::Infer(inf) => unreachable!(),
            Type::Macro(m) => todo!(),
            Type::Paren(p) => todo!(),
            Type::Path(TypePath { qself: None, path }) => path.extract_name(),

            Type::Ptr(p) => todo!(),
            _ => todo!(),
        }
    }
}

impl ExtractName for Path {
    fn extract_name(&self) -> Ident {
        self.segments.last().unwrap().ident.clone()
    }
}

fn extract_name_for_bounds<T: Traitified>(imp: &T) -> Ident {
    let name = imp.bounds().iter().filter_map(|merkmal| {
            if let TypeParamBound::Trait(trait_bound) = merkmal && matches!(trait_bound.modifier, TraitBoundModifier::None) {
                Some(&trait_bound.path)
            } else {
                None
            }
        }).fold(String::new(), | mut mock_name, path| {
            let p: String =  path.segments.iter().map(|x| format!("{}", x.ident)).collect();
            mock_name.push_str(&p);

            mock_name
        });
    Ident::new(&name, Span::call_site())
}

pub fn prepare_mock_name(name: &Ident) -> Ident {
    let inp = format!("{name}Mock");
    Ident::new(&inp, Span::call_site())
}

