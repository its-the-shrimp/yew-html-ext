use std::ops::Not;

use super::html_dashed_name::HtmlDashedName;
use super::{HtmlChildrenTree, TagTokens};
use crate::props::Prop;
use crate::{Peek, PeekValue};
use quote::{quote, quote_spanned, ToTokens};
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;

pub struct HtmlList {
    open: HtmlListOpen,
    pub children: HtmlChildrenTree,
    close: HtmlListClose,
}

impl PeekValue<()> for HtmlList {
    fn peek(cursor: Cursor) -> Option<()> {
        HtmlListOpen::peek(cursor)
            .or_else(|| HtmlListClose::peek(cursor))
            .map(|_| ())
    }
}

impl Parse for HtmlList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if HtmlListClose::peek(input.cursor()).is_some() {
            return match input.parse::<HtmlListClose>() {
                Ok(close) => Err(syn::Error::new_spanned(
                    close.to_spanned(),
                    "this closing fragment has no corresponding opening fragment",
                )),
                Err(err) => Err(err),
            };
        }

        let open = input.parse::<HtmlListOpen>()?;
        let mut children = HtmlChildrenTree::new();
        while HtmlListClose::peek(input.cursor()).is_none() {
            children.parse_child(input)?;
            if input.is_empty() {
                return Err(syn::Error::new_spanned(
                    open.to_spanned(),
                    "this opening fragment has no corresponding closing fragment",
                ));
            }
        }

        let close = input.parse::<HtmlListClose>()?;

        Ok(Self {
            open,
            children,
            close,
        })
    }
}

pub fn generate_vlist_tokens(
    children: impl ToTokens,
    key: Option<&Prop>,
    span: proc_macro2::Span,
    tokens: &mut proc_macro2::TokenStream,
) {
    let key = if let Some(key) = key {
        let v = &key.value;
        let cfg1 = key.cfg.iter();
        let cfg2 = key.cfg.iter();
        quote_spanned! {key.value.span()=> {
            #(#[cfg(#cfg1)])*
            let x = ::std::option::Option::Some(::std::convert::Into::<::yew::virtual_dom::Key>::into(#v));
            #(
                #[cfg(#cfg2)]
                let x = ::std::option::Option::<::yew::virtual_dom::Key>::None;
            )*
            x
        }}
    } else {
        quote! { ::std::option::Option::None }
    };

    tokens.extend(quote_spanned! {span=>
        ::yew::virtual_dom::VNode::VList(
            ::yew::virtual_dom::VList::with_children(#children, #key)
        )
    });
}

impl ToTokens for HtmlList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let span = {
            let open = self.open.to_spanned();
            let close = self.close.to_spanned();
            quote! { #open #close }.span()
        };

        generate_vlist_tokens(&self.children, self.open.props.key.as_ref(), span, tokens);
    }
}

struct HtmlListOpen {
    tag: TagTokens,
    props: HtmlListProps,
}

impl HtmlListOpen {
    fn to_spanned(&self) -> impl ToTokens {
        self.tag.to_spanned()
    }
}

impl PeekValue<()> for HtmlListOpen {
    fn peek(cursor: Cursor) -> Option<()> {
        let (punct, cursor) = cursor.punct()?;
        if punct.as_char() != '<' {
            return None;
        }
        // make sure it's either a property (key=value) or it's immediately closed
        if let Some((_, cursor)) = HtmlDashedName::peek(cursor) {
            matches!(cursor.punct()?.0.as_char(), '=' | '?')
                .not()
                .then_some(())
        } else {
            matches!(cursor.punct()?.0.as_char(), '>').then_some(())
        }
    }
}

impl Parse for HtmlListOpen {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        TagTokens::parse_start_content(input, |input, tag| {
            let props = input.parse()?;
            Ok(Self { tag, props })
        })
    }
}

struct HtmlListProps {
    key: Option<Prop>,
}
impl Parse for HtmlListProps {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = if input.is_empty() {
            None
        } else {
            let prop: Prop = input.parse()?;
            if !input.is_empty() {
                return Err(input.error("only a single `key` prop is allowed on a fragment"));
            }

            if prop.label.to_ascii_lowercase_string() != "key" {
                return Err(syn::Error::new_spanned(
                    prop.label,
                    "fragments only accept the `key` prop",
                ));
            }

            Some(prop)
        };

        Ok(Self { key })
    }
}

struct HtmlListClose(TagTokens);
impl HtmlListClose {
    fn to_spanned(&self) -> impl ToTokens {
        self.0.to_spanned()
    }
}

impl PeekValue<()> for HtmlListClose {
    fn peek(cursor: Cursor) -> Option<()> {
        let (_, cursor) = cursor.punct().filter(|(punct, _)| punct.as_char() == '<')?;
        let (_, cursor) = cursor.punct().filter(|(punct, _)| punct.as_char() == '/')?;
        cursor
            .punct()
            .map_or(false, |(punct, _)| punct.as_char() == '>')
            .then_some(())
    }
}

impl Parse for HtmlListClose {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        TagTokens::parse_end_content(input, |input, tag| {
            if !input.is_empty() {
                Err(input.error("unexpected content in list close"))
            } else {
                Ok(Self(tag))
            }
        })
    }
}
