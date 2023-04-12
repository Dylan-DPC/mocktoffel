use proc_macro::TokenStream;
use syn::{Field, Type, TypePath};

pub struct MockPrepared {
    pub mocked_type: Type,
    pub stream: TokenStream,
}

impl MockPrepared {
    pub fn new(mocked_type: Type, stream: TokenStream) -> Self {
        Self {
            mocked_type,
            stream,
        }
    }
}

pub fn mutate(tokens: &mut Field, new: MockPrepared) {
    if let Some(k) = &tokens.ident {
        if let Type::Path(p) = &mut tokens.ty && let Type::Path(TypePath { path: m, ..}) = new.mocked_type {
            *p = TypePath { qself: p.qself.clone(), path: m.clone()};
}
    } else {
        todo!()
    }
}
