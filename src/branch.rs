use proc_macro::TokenStream;
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

pub fn resolve_path_and_mock(p: &TypePath) -> MockPrepared {
    let segments: Vec<&PathSegment> = p.path.segments.iter().collect();
    match &segments[..] {
        &[x] => {
            let mut new_class = parse_str(format!("{}Mock", x.ident).as_str()).unwrap();
            if let Type::Path(ref mut p) = new_class {
                let segment = p.path.segments.last_mut().unwrap();
                segment.arguments = x.arguments.clone();
            }

            MockPrepared::new(new_class, None)
        }
        _ => todo!(),
    }
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
