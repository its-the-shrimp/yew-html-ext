//! This crate provides handy extensions to [Yew](https://yew.rs)'s
//! [HTML macros](https://docs.rs/yew/latest/yew/macro.html.html).
//! It provides [`html!`] and [`html_nested!`] macros that are fully backwards-compatible with the
//! original ones defined in Yew, meaning all one has to do to start using this crate is
//! just change the uses/imports of `yew::html{_nested}` to `yew_html_ext::html{_nested}`.
//! # New syntax
//! ## `for` loops
//! The syntax is the same as of Rust's `for` loops, the body of the loop can contain 0 or more
//! nodes.
//! ```rust
//! use yew_html_ext::html;
//! use yew::{Properties, function_component, html::Html};
//!
//! #[derive(PartialEq, Properties)]
//! struct CountdownProps {
//!     n: usize,
//! }
//!
//! #[function_component]
//! fn Countdown(props: &CountdownProps) -> Html {
//!     html! {
//!         <div>
//!             for i in (0 .. props.n).rev() {
//!                 <h2>{ i }</h2>
//!                 <br />
//!             }
//!         </div>
//!     }
//! }
//! ```
//! In a list of nodes all nodes must have unique keys or have no key, which is why using a
//! constant to specify a key of a node in a loop is dangerous: if the loop iterates more than
//! once, the generated list will have repeated keys; as a best-effort attempt to prevent such
//! cases, the macro disallows specifying literals or constants as keys
//! ```rust,compile_fail
//! # use yew::{Properties, function_component, html::Html};
//! # use yew_html_ext::html;
//! #
//! # #[derive(PartialEq, Properties)]
//! # struct CountdownProps {
//! #     n: usize,
//! # }
//! #
//! # #[function_component]
//! # fn Countdown(props: &CountdownProps) -> Html {
//! html! {
//!     <div>
//!         for i in (0 .. props.n).rev() {
//!             <h2 key="number" /* nuh-uh */>{ i }</h2>
//!             <br />
//!         }
//!     </div>
//! }
//! # }
//! ```
//! ## `match` nodes
//! The syntax is the same as of Rust's `match` expressions; the body of a match arm must have
//! exactly 1 node.
//! ```rust
//! use yew_html_ext::html;
//! use yew::{Properties, function_component, html::Html};
//! use std::cmp::Ordering;
//!
//! #[derive(PartialEq, Properties)]
//! struct ComparisonProps {
//!     int1: usize,
//!     int2: usize,
//! }
//!
//! #[function_component]
//! fn Comparison(props: &ComparisonProps) -> Html {
//!     html! {
//!         match props.int1.cmp(&props.int2) {
//!             Ordering::Less => { '<' },
//!             Ordering::Equal => { '=' },
//!             Ordering::Greater => { '>' },
//!         }
//!     }
//! }
//! ```
//! ## `let` bindings
//! Normal Rust's `let` bindings, including `let-else` structures, are supported with the same
//! syntax.
//! ```rust
//! use yew_html_ext::html;
//! use yew::{Properties, function_component, html::Html};
//! use std::{fs::read_dir, path::PathBuf};
//!
//! #[derive(PartialEq, Properties)]
//! struct DirProps {
//!     path: PathBuf,
//! }
//!
//! #[function_component]
//! fn Dir(props: &DirProps) -> Html {
//!     html! {
//!         <ul>
//!             let Ok(iter) = read_dir(&props.path) else {
//!                 return html!("oops :P")
//!             };
//!             for entry in iter {
//!                 let Ok(entry) = entry else {
//!                     return html!("oops :p")
//!                 };
//!                 <li>{ format!("{:?}", entry.path()) }</li>
//!             }
//!         </ul>
//!     }
//! }
//! ```
//! ## `#[cfg]` on props of elements & components
//! Any number of `#[cfg]` attributes can be applied to any prop of a of an element or component.
//!
//! ```rust
//! use yew_html_ext::html;
//! use yew::{function_component, Html};
//!
//! #[function_component]
//! fn DebugStmt() -> Html {
//!     html! {
//!         <code #[cfg(debug_assertions)] style="color: green">
//!             { "Make sure this is not green" }
//!         </code>
//!     }
//! }
//! ```

mod html_tree;
mod props;
mod stringify;

use html_tree::{HtmlRoot, AsVNode};
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::buffer::Cursor;
use syn::parse_macro_input;

trait OptionExt<T, U> {
    fn unzip_ref(&self) -> (Option<&T>, Option<&U>);
}

impl<T, U> OptionExt<T, U> for Option<(T, U)> {
    fn unzip_ref(&self) -> (Option<&T>, Option<&U>) {
        if let Some((x, y)) = self {
            (Some(x), Some(y))
        } else {
            (None, None)
        }
    }
}

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

#[proc_macro_error2::proc_macro_error]
#[proc_macro]
pub fn html_nested(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as HtmlRoot);
    TokenStream::from(root.into_token_stream())
}

#[proc_macro_error2::proc_macro_error]
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as AsVNode<HtmlRoot>);
    TokenStream::from(root.into_token_stream())
}
