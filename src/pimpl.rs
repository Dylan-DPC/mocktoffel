use std::ops::Deref;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_str, FnArg, ImplItem, ImplItemFn, ItemImpl, ReturnType, Type, TypePath};

use crate::extract::{prepare_mock_name, ExtractName, Extracted};

pub struct MockContext {
    original_type: Box<Type>,
    mocked_extract: Extracted,
}

impl MockContext {
    pub fn new(original_type: Box<Type>) -> Self {
        let mock_name = prepare_mock_name(&original_type.extract_name());
        Self {
            original_type,
            mocked_extract: mock_name,
        }
    }

    pub fn mock_impl(&self, tokens: ItemImpl) -> TokenStream {
        let original_type = tokens.self_ty.clone();
        let original_name = original_type.extract_name();
        let Extracted { name, .. } = prepare_mock_name(&original_name);

        match &tokens.trait_ {
            Some((_, tr, _)) => tokens
                .items
                .into_iter()
                .map(|item| {
                    let functions = match item {
                        ImplItem::Fn(mut f) => self.replace_self_from_function_with_mocks(f),
                        _ => todo!(),
                    };

                    let Extracted {
                        name: trait_,
                        generics: trait_generics,
                    } = tr.extract_name();
                    let impl_generics = tokens.generics.clone();

                    TokenStream::from(quote! {
                        impl #impl_generics #trait_ #trait_generics for #name {
                            #functions
                        }
                    })
                })
                .collect(),
            _ => tokens
                .items
                .into_iter()
                .map(|item| {
                    let functions = match item {
                        ImplItem::Fn(mut f) => self.replace_self_from_function_with_mocks(f),
                        _ => todo!(),
                    };

                    let impl_generics = tokens.generics.clone();
                    let generics = original_name.generics.clone();

                    TokenStream::from(quote! {
                        impl #impl_generics #name #generics {
                            #functions
                        }
                    })
                })
                .collect(),
        }
    }

    #[allow(clippy::explicit_deref_methods)]
    fn replace_self_from_function_with_mocks(&self, mut f: ImplItemFn) -> ImplItemFn {
        let visibility = f.vis;

        f.sig.inputs.iter_mut().filter_map(|arg| {
                      if let FnArg::Typed(ref mut typ) = arg && typ.ty == self.original_type && let Type::Path(ref mut p) = &mut *typ.ty {
                          Some(p)
                      } else {
                          None
                      }
              }).for_each(|p| {
                    let segments = p.path.segments.last_mut().unwrap();
                    let fn_generics = segments.arguments.clone();
                    *p = parse_str(format!("{}", self.mocked_extract.name).as_str()).unwrap();

                        let segment = p.path.segments.last_mut().unwrap();
                        segment.arguments = fn_generics;
              });

        let inputs = f.sig.inputs.iter();

        let function_name = f.sig.ident;
        let name = self.mocked_extract.name.clone();
        let original_name = self.original_type.extract_name();
        let generics = original_name.generics;
        syn::parse(match &f.sig.output {
               ReturnType::Type(_, ref p) if let Type::Path(TypePath { path: pat, .. }) = p.deref() && pat.extract_name().name == original_name.name => {
                   if pat.extract_name().name == original_name.name {
                       TokenStream::from(quote! {
                           #visibility fn #function_name (#(#inputs),*) -> #name #generics {
                               Default::default()
                           }
                       })
                   } else {
                      TokenStream::from(quote!(f))
                   }
               },

               ReturnType::Type(_, ty) => {
                   TokenStream::from(quote!{
                       #visibility fn #function_name(#(#inputs), *) -> #ty {
                           Default::default()
                       }
                   })
               },

               ReturnType::Default => {
                   TokenStream::from(quote!{
                       #visibility fn #function_name(#(#inputs), *) {
                       }
                   })
               }
           }).unwrap()
    }
}
