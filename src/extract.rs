use crate::branch::Traitified;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::fmt::Write;
use std::iter::FilterMap;
use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, GenericParam, Generics, Ident, Item,
    ItemEnum, ItemStruct, Meta, Path, PathArguments, TraitBoundModifier, Type, TypeParam,
    TypeParamBound, TypePath,
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
            let p: String = path.segments.iter().fold(String::new(), |mut full_bound, segment| {
                let _ = write!(full_bound, "{}", segment.ident);
               full_bound
            });

            mock_name.push_str(&p);

            mock_name
        });
    Ident::new(&name, Span::call_site())
}

pub fn prepare_mock_name(name: &Extracted) -> Extracted {
    let inp = format!("{}Mock", name.name);
    Extracted::new(Ident::new(&inp, Span::call_site()), name.generics.clone())
}

pub fn parse_fields_and_generate_for_values(schtruct: &mut ItemStruct) -> TokenStream {
    let (fields, values) =
        schtruct
            .fields
            .iter()
            .fold((vec![], vec![]), |(mut fields, mut values), field| {
                match (field.attrs.iter().find(|attr| attr.meta.path().is_ident("mocked_with")), &field.ident) {
            (Some(v), Some(ref ident )) if let Meta::List(ref list) = v.meta => {
                fields.push(ident.clone());
                values.push(list.tokens.clone());
            },
            (Some(v), _) if let Meta::List(ref list) = v.meta => {
                values.push(list.tokens.clone());
            },
            (None, Some(ref ident)) => {
                fields.push(ident.clone());
                values.push(quote!( { Default::default() }));

            },
            (None, _) => {
                values.push(quote!( { Default::default() }));
            },
            _ => todo!(),
        };

                (fields, values)
            });

    let struct_name = &schtruct.ident;
    let generics = &mut schtruct.generics;
    let mut impl_generics = generics.clone();
    extract_generics_from_bounds(&mut impl_generics);

    let tok = match (&fields[..], &values[..]) {
        (&[], &[]) => {
            quote! {
                impl #generics #struct_name #impl_generics {
                    pub fn mock_new() -> Self {
                        Self {}
                    }
                }
            }
        }
        (&[], v) => {
            quote! {
                impl #generics #struct_name #impl_generics {
                    pub fn mock_new() -> Self {
                        Self(#(#values),*)
                    }
                }
            }
        }
        (f, v) => {
            quote! {
                impl #impl_generics #struct_name #impl_generics {
                    pub fn mock_new() -> Self {
                        Self {
                            #(#fields : #values),*
                        }
                    }
                }
            }
        }
    };
    TokenStream::from(tok)
}

#[allow(clippy::option_if_let_else)]
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

pub fn extract_generics_from_bounds(bounds: &mut Generics) {
    bounds.params.iter_mut().for_each(|bound| match bound {
        GenericParam::Type(ty) => {
            ty.colon_token = None;
            ty.bounds = Punctuated::new();
        }
        GenericParam::Const(c) => todo!(),
        GenericParam::Lifetime(lp) => {}
    });
}

pub fn clean_out_attributes(item: &mut Item) {
    match item {
        Item::Struct(s) => {
            s.fields.iter_mut().for_each(|field| {
                field.attrs = field
                    .attrs
                    .extract_if(|attr| {
                        if let Meta::Path(ref p) = attr.meta {
                            p.get_ident().map(|x| x.to_string() == "mocked").is_some()
                        } else {
                            false
                        }
                    })
                    .collect();
            });
        }
        Item::Enum(e) => {
            e.variants.iter_mut().for_each(|field| {
                field.attrs = field
                    .attrs
                    .extract_if(|attr| {
                        if let Meta::Path(ref p) = attr.meta {
                            p.get_ident().map(|x| x.to_string() == "mocked").is_some()
                        } else {
                            false
                        }
                    })
                    .collect();
            });
        }
        _ => unreachable!(),
    }
}
