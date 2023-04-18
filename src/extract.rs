use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream, Result},
    ItemEnum, ItemStruct, Token, Type,
};

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

#[derive(Clone)]
pub enum MockableType {
    Struct(ItemStruct),
    Enum(ItemEnum),
}

impl Parse for MockableType {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();
        if la.peek(Token![struct]) {
            input.parse().map(Self::Struct)
        } else if la.peek(Token![enum]) {
            input.parse().map(Self::Enum)
        } else {
            todo!()
        }
    }
}

impl ToTokens for MockableType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            MockableType::Struct(its) => its.to_tokens(tokens),
            MockableType::Enum(ite) => ite.to_tokens(tokens),
        }
    }
}
