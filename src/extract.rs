use crate::branch::Traitified;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::iter::FilterMap;
use syn::{
    AngleBracketedGenericArguments, Ident, Item, ItemEnum, ItemStruct, Meta, Path, PathArguments,
    TraitBoundModifier, Type, TypeParamBound, TypePath,
};

pub struct MockPrepared {
    pub mocked_type: Type,
    stream: Option<TokenStream>,
}

impl MockPrepared {
    pub const fn new(mocked_type: Type, stream: Option<TokenStream>) -> Self {
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
    pub const fn new(name: Ident, generics: Option<AngleBracketedGenericArguments>) -> Self {
        Self { name, generics }
    }

    pub const fn with_ident(name: Ident) -> Self {
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
            Self::Array(_) => todo!(),
            Self::BareFn(f) => todo!(),
            Self::Group(g) => todo!(),
            Self::ImplTrait(im) => Extracted::with_ident(extract_name_for_bounds(im)),
            Self::Infer(inf) => unreachable!(),
            Self::Macro(m) => todo!(),
            Self::Paren(p) => todo!(),
            Self::Path(TypePath { qself: None, path }) => path.extract_name(),
            Self::Ptr(p) => todo!(),
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

pub fn parse_fields_and_generate_for_values(schtruct: &ItemStruct) -> TokenStream {
    let (fields, values)  = schtruct.fields.iter().fold((vec![], vec![]), |(mut fields, mut values), field | {
        match field.attrs.iter().find(|attr| attr.meta.path().is_ident("mock_with")) {
            Some(v) if let Meta::List(ref list) = v.meta && let Some(value) = list.tokens.clone().into_iter().next() && let Some(ref ident) = field.ident => {
                fields.push(ident.clone());
                values.push(value);
            },
            None => {},
            _ => todo!(),
        };

        (fields, values)
    });

    let struct_name = schtruct.ident.clone();
    let tok = if fields.is_empty() {
        quote! {
            impl #struct_name {
                pub fn mock_new() -> Self {
                        Self::default()
                    }
                }
        }
    } else {
        quote! {
            impl #struct_name {
                pub fn mock_new() -> Self {
                    Self {
                        #(#fields : #values),*
                    }
                }
                }
        }
    };
    TokenStream::from(tok)
}

pub fn parse_fields_and_generate_variant(enoom: &ItemEnum) -> TokenStream {
    let enum_name = enoom.ident.clone();

    let tok = if let Some(variant) = enoom.variants.iter().find(|field| {
        field
            .attrs
            .iter()
            .any(|attr| attr.meta.path().is_ident("mocked_value"))
    }) {
        let variant_name = &variant.ident;
        quote! {
            impl #enum_name {
                pub fn mock_new() -> Self {
                    Self::#variant_name
                }
            }
        }
    } else if let Some(variant) = enoom.variants.iter().find(|field| {
        field
            .attrs
            .iter()
            .any(|attr| attr.meta.path().is_ident("mocked_value_with"))
    }) {
        let variant_name = &variant.ident;
        let value = if let Meta::List(ref ml) = variant.attrs.first().unwrap().meta {
            &ml.tokens
        } else {
            unreachable!()
        };

        quote! {
            impl #enum_name {
                pub fn mock_new() -> Self {
                    Self::#variant_name(#value)
                }
            }
        }
    } else {
        quote! {
            impl #enum_name {
                pub fn mock_new() -> Self {
                   Self::default()
                }
            }
        }
    };

    TokenStream::from(tok)
}

pub fn clean_out_attributes(item: &mut Item) {
    match item {
        Item::Struct(s) => {
            s.fields.iter_mut().for_each(|field| {
                field.attrs.clear();
            });
        }
        Item::Enum(e) => {
            e.variants.iter_mut().for_each(|variant| {
                variant.attrs.clear();
            });
        }
        _ => unreachable!(),
    }
}
