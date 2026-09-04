use crate::extract::{AttributeCleaner, PrepareMockParams};
use proc_macro::TokenStream;
use quote::quote;
use syn::Item;
pub fn mock(mut tokens: Item) -> TokenStream {
    let mut mock = tokens.clone();
    let fields = match mock {
        Item::Struct(ref mut s) => s.extract_mock(),
        Item::Enum(ref mut e) => e.extract_mock(),
        _ => todo!(),
    };
    tokens.clean_out_attributes();
    mock.clean_out_attributes();
    let mut original_and_mocked_struct = TokenStream::from(quote! {
        #tokens
        #mock
    });

    original_and_mocked_struct.extend_one(fields);
    original_and_mocked_struct
}
