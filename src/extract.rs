use proc_macro::TokenStream;
use syn::Type;

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
