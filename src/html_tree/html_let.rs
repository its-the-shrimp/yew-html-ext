use crate::PeekValue;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    buffer::Cursor,
    parse::{Parse, ParseStream},
    Local, LocalInit, Pat, PatType,
};

pub struct HtmlLet(Local);

impl PeekValue<()> for HtmlLet {
    fn peek(cursor: Cursor) -> Option<()> {
        let (ident, _) = cursor.ident()?;
        (ident == "let").then_some(())
    }
}

impl Parse for HtmlLet {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let let_token = input.parse()?;

        let mut pat = Pat::parse_single(input)?;
        if let Some(colon_token) = input.parse()? {
            pat = Pat::Type(PatType {
                attrs: vec![],
                pat: Box::new(pat),
                colon_token,
                ty: input.parse()?,
            });
        }

        let init = if let Some(eq_token) = input.parse()? {
            Some(LocalInit {
                eq_token,
                expr: input.parse()?,
                diverge: if let Some(else_token) = input.parse()? {
                    Some((else_token, input.parse()?))
                } else {
                    None
                },
            })
        } else {
            None
        };

        Ok(Self(Local {
            attrs: vec![],
            let_token,
            pat,
            init,
            semi_token: input.parse()?,
        }))
    }
}

impl ToTokens for HtmlLet {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }

    fn to_token_stream(&self) -> TokenStream {
        self.0.to_token_stream()
    }

    fn into_token_stream(self) -> TokenStream {
        self.0.into_token_stream()
    }
}
