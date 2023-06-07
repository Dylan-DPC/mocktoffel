use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_str, punctuated::Punctuated, token::Plus, Ident, PathSegment, TraitBoundModifier, Type,
    TypeBareFn, TypeImplTrait, TypeParamBound, TypePath, TypePtr, TypeReference, TypeTraitObject,
    TypeTuple,
};

use crate::extract::MockPrepared;

pub fn get_mocking_candidate(field: &Type) -> MockPrepared {
    match field {
        Type::Array(arr) => get_mocking_candidate(&arr.elem),
        Type::BareFn(f) => mock_function(f),
        Type::Group(g) => get_mocking_candidate(&g.elem),
        Type::ImplTrait(imp) => mock_and_impl_trait_for_it(imp),
        Type::Infer(_) => unreachable!(),
        //TODO:: consider if we need to consider this case or not
        Type::Macro(_) => todo!(),
        Type::Never(_) => todo!(),
        Type::Paren(p) => get_mocking_candidate(&p.elem),
        Type::Path(
            p @ TypePath {
                qself: Some(q),
                path: _,
            },
        ) => mock_associated_type(p),
        Type::Path(p) => resolve_path_and_mock(p),
        Type::Ptr(p) => mock_pointer(p),
        Type::Reference(r) => mock_reference(r),
        Type::Slice(sl) => get_mocking_candidate(&sl.elem),
        Type::TraitObject(dym) => mock_and_impl_trait_for_it(dym),
        Type::Tuple(tup) => mock_tuple(tup),
        Type::Verbatim(ts) => todo!(),
        _ => unreachable!(),
    }
}

pub fn mock_function(f: &TypeBareFn) -> MockPrepared {
    todo!()
}

pub fn mock_and_impl_trait_for_it<T: Traitified>(imp: &T) -> MockPrepared {
    let bounds = imp.bounds().iter().filter_map(|merkmal| {
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

    let ident = create_mock(&bounds);
    MockPrepared::new(todo!(), Some(ident))
}

pub fn create_mock(mock_name: &str) -> TokenStream {
    let stream = quote! {
        struct mock_name {}
    };

    TokenStream::from(stream)
}

pub fn implement_trait_for_mock(tr: Ident) -> TokenStream {
    todo!()
}

pub fn mock_pointer(p: &TypePtr) -> MockPrepared {
    todo!()
}

pub fn mock_associated_type(p: &TypePath) -> MockPrepared {
    todo!()
}

pub fn mock_reference(r: &TypeReference) -> MockPrepared {
    todo!()
}

pub fn resolve_path_and_mock(path: &TypePath) -> MockPrepared {
    let mut path = path.clone();
    let segment = path.path.segments.last_mut().unwrap();
    segment.ident = Ident::new(format!("{}Mock", segment.ident).as_str(), Span::call_site());
    MockPrepared::new(Type::Path(path), None)
}

pub fn mock_tuple(t: &TypeTuple) -> MockPrepared {
    todo!()
}

pub trait Traitified {
    fn bounds(&self) -> &Punctuated<TypeParamBound, Plus>;
}

impl Traitified for TypeImplTrait {
    fn bounds(&self) -> &Punctuated<TypeParamBound, Plus> {
        &self.bounds
    }
}
impl Traitified for TypeTraitObject {
    fn bounds(&self) -> &Punctuated<TypeParamBound, Plus> {
        &self.bounds
    }
}

mod tests {
    use super::*;

    #[test]
    pub fn resolve_path_and_mock_for_single_segment() {
        let path = syn::TypePath {
            qself: None,
            path: syn::parse_str("Foo").unwrap(),
        };
        let mocked = resolve_path_and_mock(&path);
        assert_eq!(mocked.mocked_type, syn::parse_str("FooMock").unwrap());
    }

    #[test]
    pub fn resolve_multi_segment() {
        let path = syn::TypePath {
            qself: None,
            path: syn::parse_str("qow::Fow").unwrap(),
        };
        let mocked = resolve_path_and_mock(&path);
        assert_eq!(mocked.mocked_type, syn::parse_str("qow::FowMock").unwrap());
    }
}
