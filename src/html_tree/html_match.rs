use super::HtmlTree;
use crate::PeekValue;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    buffer::Cursor,
    parse::{Parse, ParseStream},
    Expr, Pat, Token,
};

pub struct HtmlMatchArm {
    pat: Box<Pat>,
    guard: Option<Box<Expr>>,
    body: Box<HtmlTree>,
}

impl Parse for HtmlMatchArm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pat = Box::new(Pat::parse_multi_with_leading_vert(input)?);
        let guard = match <Token![if]>::parse(input) {
            Ok(_) => Some(input.parse()?),
            Err(_) => None,
        };
        <Token![=>]>::parse(input)?;
        let body = input.parse()?;
        Ok(Self { pat, guard, body })
    }
}

impl ToTokens for HtmlMatchArm {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { pat, guard, body } = self;
        let guard = guard.as_ref().into_iter();
        tokens.extend(quote! {
            #pat #(if #guard)* => ::std::convert::Into::<::yew::virtual_dom::VNode>::into(#body),
        })
    }
}

pub struct HtmlMatch {
    expr: Box<Expr>,
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
        <Token![match]>::parse(input)?;
        let expr = Box::new(Expr::parse_without_eager_brace(input)?);
        let body;
        braced!(body in input);
        let arms = body
            .parse_terminated(HtmlMatchArm::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(Self { expr, arms })
    }
}

impl ToTokens for HtmlMatch {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { expr, arms } = self;
        tokens.extend(quote! {
            match #expr {
                #(#arms)*
            }
        })
    }
}
