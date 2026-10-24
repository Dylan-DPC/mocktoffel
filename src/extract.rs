use crate::branch::Traitified;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::fmt::Write;
use syn::{
    AngleBracketedGenericArguments, Expr, GenericParam, Generics, Ident, Item, ItemEnum, ItemFn,
    ItemStruct, Meta, Path, PathArguments, ReturnType, Signature, TraitBoundModifiers, Type,
    TypeParamBound, TypePath, punctuated::Punctuated,
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
pub trait MockExtractor {
    fn extract_name(&self) -> Extracted;
}

impl MockExtractor for Type {
    fn extract_name(&self) -> Extracted {
        match self {
            Self::Array(_) => todo!(),
            Self::FnPtr(f) => todo!(),
            Self::Group(g) => todo!(),
            Self::ImplTrait(im) => Extracted::with_ident(extract_name_for_bounds(im)),
            Self::Infer(inf) => unreachable!(),
            Self::Macro(m) => todo!(),
            Self::Paren(p) => todo!(),
            Self::Path(TypePath {
                qself: None,
                path,
                attrs,
            }) => path.extract_name(),
            Self::Ptr(p) => todo!(),
            _ => todo!(),
        }
    }
}

impl MockExtractor for Path {
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
    let name = imp
        .bounds()
        .iter()
        .filter_map(|merkmal| {
            if let TypeParamBound::Trait(trait_bound) = merkmal
                && trait_bound.modifiers == TraitBoundModifiers::default()
            {
                Some(&trait_bound.path)
            } else {
                None
            }
        })
        .fold(String::new(), |mut mock_name, path| {
            let p: String = path
                .segments
                .iter()
                .fold(String::new(), |mut full_bound, segment| {
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

pub fn get_mocked_value_from_attributes(enoom: &mut ItemEnum, mocked: &Extracted) -> Option<Expr> {
    let name = &enoom.ident;
    enoom
        .attrs
        .iter_mut()
        .find_map(|attr| match &mut attr.meta {
            Meta::NameValue(nv)
                if let Some(_) = nv.path.get_ident().map(|x| *x == "mocked_with") =>
            {
                replace_with_mocked(&mut nv.value, name, mocked);
                let path = &nv.value;
                Some(syn::parse(TokenStream::from(quote!(<#path>))).unwrap())
            }
            Meta::NameValue(nv)
                if let Some(_) = nv.path.get_ident().map(|x| *x == "mocked_with_default") =>
            {
                Some(syn::parse(TokenStream::from(quote!(<#name>::default()))).unwrap())
            }

            _ => None,
        })
}

pub trait AttributeCleaner {
    fn clean_out_attributes(&mut self);
}

impl AttributeCleaner for Item {
    fn clean_out_attributes(&mut self) {
        match self {
            Item::Struct(s) => s.clean_out_attributes(),
            Item::Enum(e) => e.clean_out_attributes(),
            Item::Fn(f) => f.clean_out_attributes(),
            _ => {}
        }
    }
}

impl AttributeCleaner for ItemStruct {
    fn clean_out_attributes(&mut self) {
        self.fields.iter_mut().for_each(|field| {
            field.attrs = field
                .attrs
                .extract_if(.., |attr| {
                    if let Meta::Path(ref p) = attr.meta {
                        dbg!(&p);
                        p.get_ident()
                            .map(|x| *x == "mocked" || *x == "mocked_with")
                            .is_some()
                    } else {
                        false
                    }
                })
                .collect();
        });
        dbg!(&self.fields);
    }
}

impl AttributeCleaner for ItemEnum {
    fn clean_out_attributes(&mut self) {
        self.attrs.retain(|attr| {
            if let Meta::NameValue(p) = &attr.meta
                && p.path.get_ident().map(|x| *x == "mocked_with").is_some()
            {
                false
            } else {
                true
            }
        });

        self.variants.iter_mut().for_each(|field| {
            field.attrs = field
                .attrs
                .iter()
                .filter_map(|attr| {
                    if let Meta::NameValue(ref p) = attr.meta
                        && p.path.get_ident().map(|x| *x == "mocked_with").is_some()
                    {
                        None
                    } else {
                        Some(attr.clone())
                    }
                })
                .collect();
        });
    }
}

impl AttributeCleaner for ItemFn {
    fn clean_out_attributes(&mut self) {
        self.attrs = self
            .attrs
            .extract_if(.., |attr| {
                if let Meta::Path(ref p) = attr.meta {
                    p.get_ident().map(|x| *x == "mocked").is_some()
                } else {
                    false
                }
            })
            .collect();
    }
}

fn replace_with_mocked(expr: &mut Expr, name: &Ident, mocked: &Extracted) {
    match expr {
        Expr::Path(path) => {
            let mocked_segment = path
                .path
                .segments
                .iter_mut()
                .find(|path| path.ident == *name)
                .unwrap();
            mocked_segment.ident = mocked.name.clone();
        }

        Expr::Call(call) => {
            if let Expr::Path(p) = call.func.as_mut() {
                let mocked_segment = p
                    .path
                    .segments
                    .iter_mut()
                    .find(|path| path.ident == *name)
                    .unwrap();
                mocked_segment.ident = mocked.name.clone();
            }
        }

        _ => todo!("other exprs matter"),
    }
}

pub trait PrepareMockParams: AttributeCleaner {
    fn extract_mock(&mut self) -> TokenStream {
        let name = self.mock_name();
        self.set_ident(name);
        self.extract_members()
    }
    fn mock_name(&self) -> Ident;
    fn extract_members(&mut self) -> TokenStream;
    fn set_ident(&mut self, ident: Ident);
}

impl PrepareMockParams for ItemStruct {
    fn mock_name(&self) -> Ident {
        Ident::new(format!("{}Mock", self.ident).as_str(), Span::call_site())
    }

    fn extract_members(&mut self) -> TokenStream {
        let (fields, values) =
            self.fields
                .iter()
                .fold((vec![], vec![]), |(mut fields, mut values), field| {
                    match (
                        field
                            .attrs
                            .iter()
                            .find(|attr| attr.meta.path().is_ident("mocked_with")),
                        &field.ident,
                    ) {
                        (Some(v), Some(ident)) if let Meta::List(ref list) = v.meta => {
                            fields.push(ident.clone());
                            values.push(list.tokens.clone());
                        }
                        (Some(v), _) if let Meta::List(ref list) = v.meta => {
                            values.push(list.tokens.clone());
                        }
                        (None, Some(ident)) => {
                            fields.push(ident.clone());
                            values.push(quote!({ Default::default() }));
                        }
                        (None, _) => {
                            values.push(quote!({ Default::default() }));
                        }
                        _ => todo!(),
                    }

                    (fields, values)
                });

        let struct_name = &self.ident;
        let generics = &self.generics;
        let mut impl_generics = self.generics.clone();
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

    fn set_ident(&mut self, ident: Ident) {
        self.ident = ident;
    }
}

impl PrepareMockParams for ItemEnum {
    fn mock_name(&self) -> Ident {
        Ident::new(format!("{}Mock", self.ident).as_str(), Span::call_site())
    }

    fn extract_members(&mut self) -> TokenStream {
        let enum_name = self.ident.clone();
        let extracted = Extracted::with_ident(enum_name.clone());
        let mocked = prepare_mock_name(&extracted);

        let mocked_name = &mocked.name;

        let tok = if let Some(variant) = self.variants.iter_mut().find(|field| {
            field.attrs.iter().any(|attr| {
                if let Meta::NameValue(nv) = &attr.meta
                    && nv.path.is_ident("mocked_with")
                {
                    true
                } else {
                    false
                }
            })
        }) {
            let variant_name = &mut variant.ident;
            if let Meta::NameValue(ref mut ml) = variant.attrs.first_mut().unwrap().meta {
                let value = &mut ml.value;
                replace_with_mocked(value, &enum_name, &mocked);
                quote! {
                    impl #enum_name {
                        pub fn mock_new() -> #mocked_name {
                            #value
                        }
                    }
                }
            } else {
                quote! {
                    impl #enum_name {
                        pub fn mock_new() -> #mocked_name {
                            Self::#variant_name
                        }
                    }
                }
            }
        } else if let (Some(variant), Some(field)) =
            self.variants
                .iter()
                .fold((None, None), |(picked_value, picked_field), variant| {
                    variant.fields.iter().fold(
                        (picked_value, picked_field),
                        |(picked_value, picked_field), field| {
                            field.attrs.iter().fold(
                                (picked_value, picked_field),
                                |(picked_value, field), attr| {
                                    if let Meta::NameValue(nv) = &attr.meta
                                        && nv.path.is_ident("mocked_with")
                                    {
                                        (Some(variant), Some(nv.value.clone()))
                                    } else {
                                        (None, None)
                                    }
                                },
                            )
                        },
                    )
                })
        {
            let variant_name = &variant.ident;
            quote! {
                impl #enum_name {
                    pub fn mock_new() -> #mocked_name {
                        Self::#variant_name(#field)
                    }
                }
            }
        } else if let Some(mocked_value) = get_mocked_value_from_attributes(self, &mocked) {
            quote! {
                impl #enum_name {
                    pub fn mock_new() -> #mocked_name {
                        #mocked_value
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

    fn set_ident(&mut self, ident: Ident) {
        self.ident = ident;
    }
}

impl PrepareMockParams for ItemFn {
    fn mock_name(&self) -> Ident {
        Ident::new(
            format!("{}_mock", self.sig.ident).as_str(),
            Span::call_site(),
        )
    }

    fn extract_members(&mut self) -> TokenStream {
        let visibility = self.vis.clone();
        let Signature {
            constness,
            asyncness,
            safety,
            abi,
            fn_token: _,
            ident,
            generics,
            paren_token,
            inputs,
            variadic,
            output,
        } = &self.sig;
        let mocked = self.attrs.iter().find_map(|attr| {
            if let Meta::NameValue(nv) = &attr.meta
                && nv.path.is_ident("mocked_with")
            {
                Some(nv.value.clone())
            } else {
                None
            }
        });
        dbg!(&self.sig);
        match output {
            ReturnType::Type(ty, ret) => TokenStream::from(quote! {
                #visibility #constness #asyncness #safety #abi fn ident(#inputs) #ty #ret {
                    #mocked
                }
            }),
            ReturnType::Default => TokenStream::from(quote! {
                #visibility #constness #asyncness #safety #abi fn ident(#inputs) {
                    #mocked
                }
            }),
        }
    }

    fn set_ident(&mut self, ident: Ident) {
        self.sig.ident = ident;
    }
}
