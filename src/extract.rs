use crate::branch::Traitified;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    AngleBracketedGenericArguments, Ident, Path, PathArguments, TraitBoundModifier, Type,
    TypeParamBound, TypePath,
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

#[derive(Clone, Debug, PartialEq)]
pub struct Extracted {
    pub name: Ident,
    pub generics: Option<AngleBracketedGenericArguments>,
}

impl Extracted {
    pub fn new(name: Ident, generics: Option<AngleBracketedGenericArguments>) -> Self {
        Self { name, generics }
    }

    pub fn with_ident(name: Ident) -> Self {
        Self {
            name,
            generics: None,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait ExtractName {
    fn extract_name(&self) -> Extracted;
}

impl ExtractName for Type {
    fn extract_name(&self) -> Extracted {
        match self {
            Type::Array(_) => todo!(),
            Type::BareFn(f) => todo!(),
            Type::Group(g) => todo!(),
            Type::ImplTrait(im) => Extracted::with_ident(extract_name_for_bounds(im)),
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
    fn extract_name(&self) -> Extracted {
        let segment = self.segments.last().unwrap();
        match &segment.arguments {
            PathArguments::AngleBracketed(abga) => {
                Extracted::new(segment.ident.clone(), Some(abga.clone()))
            }
            PathArguments::None => {
                Extracted::with_ident(self.segments.last().unwrap().ident.clone())
            }

            PathArguments::Parenthesized(_) => unreachable!(),
        }
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

pub fn prepare_mock_name(name: &Extracted) -> Extracted {
    let inp = format!("{}Mock", name.name);
    Extracted::new(Ident::new(&inp, Span::call_site()), name.generics.clone())
}
