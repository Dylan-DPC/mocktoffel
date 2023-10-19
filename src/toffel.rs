use crate::branch::get_mocking_candidate;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Field, Fields, ItemEnum, ItemStruct};

pub trait Toffelise {
    fn replace_mocks(self) -> TokenStream;
}

impl Toffelise for ItemStruct {
    fn replace_mocks(self) -> TokenStream {
        let struct_name = self.ident;
        let generics = self.generics;
        let attributes = self.attrs;

        match self.fields {
            Fields::Named(named) => {
                let fields: Vec<Field> = named
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
                        }
                        field
                    })
                    .collect();

                TokenStream::from(quote! {
                    #(#attributes)*
                       struct #struct_name #generics {
                             #(#fields),*
                       }
                })
            }

            Fields::Unnamed(un) => {
                let fields: Vec<Field> = un
                    .unnamed
                    .into_iter()
                    .map(|mut field| {
                        if field
                            .attrs
                            .iter()
                            .any(|attr| attr.meta.path().is_ident("mocked"))
                        {
                            field.attrs = vec![];
                            field.ty = get_mocking_candidate(&field.ty).mocked_type;
                        }
                        field
                    })
                    .collect();

                TokenStream::from(quote! {
                    #(#attributes)*
                   struct #struct_name #generics(
                         #(#fields),*
                    );

                })
            }
            Fields::Unit => unreachable!(),
        }
    }
}

impl Toffelise for ItemEnum {
    fn replace_mocks(self) -> TokenStream {
        let enum_name = self.ident;
        let generics = self.generics;
        let attributes = self.attrs;

        let variants = self
            .variants
            .into_iter()
            .fold(vec![], |mut var, mut variant| {
                match variant.fields {
                    Fields::Unnamed(ref mut un) => un.unnamed.iter_mut().for_each(|f| {
                        if f.attrs
                            .iter()
                            .any(|attr| attr.meta.path().is_ident("mocked"))
                        {
                            f.ty = get_mocking_candidate(&f.ty).mocked_type;
                            f.attrs = vec![];
                        }
                    }),
                    Fields::Unit => {}
                    Fields::Named(_) => unreachable!(),
                };

                var.push(variant);
                var
            });

        TokenStream::from(quote! {
           #(#attributes)*
            enum #enum_name #generics {
                #(#variants),*
            }
        })
    }
}
