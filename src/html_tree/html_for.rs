use super::{HtmlChildrenTree, ToNodeIterator};
use crate::html_tree::HtmlTree;
use crate::PeekValue;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{braced, Expr, Pat, Token};

/// Determines if an expression is guaranteed to always return the same value anywhere.
fn is_contextless_pure(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(_) => true,
        Expr::Path(path) => path.path.get_ident().is_none(),
        _ => false,
    }
}

pub struct HtmlFor {
    pat: Pat,
    iter: Expr,
    brace: Brace,
    body: HtmlChildrenTree,
}

impl PeekValue<()> for HtmlFor {
    fn peek(cursor: Cursor) -> Option<()> {
        let (ident, _) = cursor.ident()?;
        (ident == "for").then_some(())
    }
}

impl Parse for HtmlFor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        <Token![for]>::parse(input)?;
        let pat = Pat::parse_single(input)?;
        <Token![in]>::parse(input)?;
        let iter = Expr::parse_without_eager_brace(input)?;

        let body_stream;
        let brace = braced!(body_stream in input);

        let body = HtmlChildrenTree::parse_delimited(&body_stream)?;
        // TODO: reduce nesting by using if-let guards / let-else statements once MSRV is raised
        for child in body.children.iter() {
            if let HtmlTree::Element(element) = child {
                if let Some(key) = &element.props.special.key {
                    if is_contextless_pure(&key.value) {
                        return Err(syn::Error::new(
                            key.value.span(),
                            "duplicate key for a node in a `for`-loop\nthis will create elements \
                             with duplicate keys if the loop iterates more than once",
                        ));
                    }
                }
            }
        }
        Ok(Self {
            pat,
            iter,
            brace,
            body,
        })
    }
}

impl ToTokens for HtmlFor {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            pat,
            iter,
            brace,
            body,
        } = self;
        let acc = Ident::new("__yew_v", iter.span());

        let alloc_opt = body
            .size_hint()
            .filter(|&size| size > 1) // explicitly reserving space for 1 more element is redundant
            .map(|size| quote!( #acc.reserve(#size); ));

        // optimisations unaviliable :(
        /*let vlist_gen = match body.fully_keyed() {
            Some(true) => quote! {
                ::yew::virtual_dom::VList::__macro_new(
                    #acc,
                    ::std::option::Option::None,
                    ::yew::virtual_dom::FullyKeyedState::KnownFullyKeyed
                )
            },
            Some(false) => quote! {
                ::yew::virtual_dom::VList::__macro_new(
                    #acc,
                    ::std::option::Option::None,
                    ::yew::virtual_dom::FullyKeyedState::KnownMissingKeys
                )
            },
            None => quote! {
                ::yew::virtual_dom::VList::with_children(#acc, ::std::option::Option::None)
            },
        };*/

        let body = body.children.iter().map(|child| {
            if let Some(child) = child.to_node_iterator_stream() {
                quote!( ::std::iter::Extend::extend(&mut #acc, #child) )
            } else {
                quote!( #acc.push(::std::convert::Into::into(#child)) )
            }
        });

        tokens.extend(quote_spanned!(brace.span.span()=> {
            let mut #acc = ::std::vec::Vec::<::yew::virtual_dom::VNode>::new();
            ::std::iter::Iterator::for_each(
                ::std::iter::IntoIterator::into_iter(#iter),
                |#pat| { #alloc_opt #(#body);* }
            );
            ::yew::virtual_dom::VList::with_children(#acc, ::std::option::Option::None)
        }))
    }
}
