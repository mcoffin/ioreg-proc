use syn::{bracketed, token};
use syn::parse::ParseStream;
use quote::ToTokens;
use proc_macro2::TokenStream;

pub trait ParseOptional: Sized {
    fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>>;
}

#[derive(Clone)]
pub struct LitVecSize {
    bracket_token: token::Bracket,
    pub count: syn::LitInt,
}

impl LitVecSize {
    #[inline]
    fn value(&self) -> u64 {
        self.count.value()
    }
}

impl ToTokens for LitVecSize {
    fn to_tokens(&self, stream: &mut TokenStream) {
        self.bracket_token.surround(stream, |stream| {
            stream.extend(self.count.clone().into_token_stream())
        });
    }
}

impl ParseOptional for LitVecSize {
    fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        let has_bracket = input.peek(token::Bracket);
        if !has_bracket {
            return Ok(None);
        }
        let content;
        Ok(Some(LitVecSize {
            bracket_token: bracketed!(content in input),
            count: content.parse()?,
        }))
    }
}
