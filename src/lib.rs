#![cfg_attr(nightly_yew, feature(proc_macro_span))]

//! This crate provides an extension for [Yew](https://yew.rs)'s
//! [HTML macros](https://docs.rs/yew/latest/yew/macro.html.html).
//! It provides [`html`] and [`html_nested`] macros that are fully backwards compatible with the
//! original ones defined in Yew, meaning all one has to do to start using this crate is
//! just change `yew::html` to `yew_html_ext::html` or change the way `html` & `html_nested` are
//! imported

mod html_tree;
mod props;
mod stringify;

use html_tree::{HtmlRoot, HtmlRootVNode};
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::buffer::Cursor;
use syn::parse_macro_input;

trait Peek<'a, T> {
    fn peek(cursor: Cursor<'a>) -> Option<(T, Cursor<'a>)>;
}

trait PeekValue<T> {
    fn peek(cursor: Cursor) -> Option<T>;
}

fn non_capitalized_ascii(string: &str) -> bool {
    if !string.is_ascii() {
        false
    } else if let Some(c) = string.bytes().next() {
        c.is_ascii_lowercase()
    } else {
        false
    }
}

/// Combine multiple `syn` errors into a single one.
/// Returns `Result::Ok` if the given iterator is empty
fn join_errors(mut it: impl Iterator<Item = syn::Error>) -> syn::Result<()> {
    it.next().map_or(Ok(()), |mut err| {
        for other in it {
            err.combine(other);
        }
        Err(err)
    })
}

fn is_ide_completion() -> bool {
    match std::env::var_os("RUST_IDE_PROC_MACRO_COMPLETION_DUMMY_IDENTIFIER") {
        None => false,
        Some(dummy_identifier) => !dummy_identifier.is_empty(),
    }
}

#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn html_nested(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as HtmlRoot);
    TokenStream::from(root.into_token_stream())
}

#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as HtmlRootVNode);
    TokenStream::from(root.into_token_stream())
}
