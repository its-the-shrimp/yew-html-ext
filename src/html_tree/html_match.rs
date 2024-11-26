use super::{AsVNode, HtmlTree};
use crate::{OptionExt, PeekValue};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    braced,
    buffer::Cursor,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Brace,
    Expr, Pat, Token,
};

pub struct HtmlMatchArm {
    pat: Pat,
    guard: Option<(Token![if], Expr)>,
    fat_arrow_token: Token![=>],
    body: AsVNode<HtmlTree>,
}

impl Parse for HtmlMatchArm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pat = Pat::parse_multi_with_leading_vert(input)?;
        let guard = match <Token![if]>::parse(input) {
            Ok(if_token) => Some((if_token, input.parse()?)),
            Err(_) => None,
        };
        let fat_arrow_token = input.parse()?;
        let body = input.parse()?;
        Ok(Self {
            pat,
            guard,
            fat_arrow_token,
            body,
        })
    }
}

impl ToTokens for HtmlMatchArm {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            pat,
            guard,
            fat_arrow_token,
            body,
        } = self;
        let (if_token, guard) = guard.unzip_ref();
        tokens.extend(quote! { #pat #if_token #guard #fat_arrow_token #body })
    }
}

pub struct HtmlMatch {
    match_token: Token![match],
    expr: Expr,
    brace: Brace,
    arms: Vec<HtmlMatchArm>,
}

impl PeekValue<()> for HtmlMatch {
    fn peek(cursor: Cursor) -> Option<()> {
        cursor
            .ident()
            .filter(|(ident, _)| ident == "match")
            .map(drop)
    }
}

impl Parse for HtmlMatch {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let match_token = input.parse()?;
        let expr = Expr::parse_without_eager_brace(input)?;
        let body;
        let brace = braced!(body in input);
        let arms = body
            .parse_terminated(HtmlMatchArm::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(Self {
            match_token,
            expr,
            brace,
            arms,
        })
    }
}

impl ToTokens for HtmlMatch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            match_token,
            expr,
            brace,
            arms,
        } = self;
        tokens.extend(quote_spanned! {brace.span.span()=>
            #match_token #expr {
                #(#arms),*
            }
        })
    }
}
