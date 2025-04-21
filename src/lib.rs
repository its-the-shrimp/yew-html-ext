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
//! exactly 1 node. That node may be just `{}`, which will expand to nothing.
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
//!             _ => {},
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
//! Any number of `#[cfg]` attributes can be applied to any prop of an element or component.
//!
//! ```rust
//! use yew_html_ext::html;
//! use yew::{function_component, Html};
//!
//! #[function_component]
//! fn DebugStmt() -> Html {
//!     html! {
//!         <code #[cfg(debug_assertions)] style="color: green;">
//!             { "Make sure this is not green" }
//!         </code>
//!     }
//! }
//! ```
//! ## Any number of top-level nodes is allowed
//! The limitation of only 1 top-level node per macro invocation of standard Yew is lifted.
//!
//! ```rust
//! use yew_html_ext::html;
//! use yew::{function_component, Html};
//!
//! #[function_component]
//! fn Main() -> Html {
//!     html! {
//!         <h1>{"Node 1"}</h1>
//!         <h2>{"Node 2"}</h2> // standard Yew would fail right around here
//!     }
//! }
//! ```
//! ## Optimisation: minified inline CSS
//! If the `style` attribute of an HTML element is set to a string literal, that string's contents
//! are interpreted as CSS & minified, namely, the whitespace between the rules & between the key &
//! value of a rule is removed, and a trailing semicolon is stripped.
//! ```rust
//! use yew_html_ext::html;
//! use yew::{function_component, Html};
//!
//! #[function_component]
//! fn DebugStmt() -> Html {
//!     html! {
//!         // the assigned style will be just `"color:green"`
//!         <strong style="
//!             color: green;
//!         ">{"Hackerman"}</strong>
//!     }
//! }
//! ```

mod html_tree;
mod props;
mod stringify;

use html_tree::{AsVNode, HtmlRoot};
use proc_macro::TokenStream;
use quote::ToTokens;
use std::fmt::{Display, Write};
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

/// Extension methods for treating `Display`able values like strings, without allocating the
/// strings.
///
/// Needed to check the plentiful token-like values in the impl of the macros, which are
/// `Display`able but which either correspond to multiple source code tokens, or are themselves
/// tokens that don't provide a reference to their repr.
trait DisplayExt: Display {
    /// Equivalent to [`str::eq_ignore_ascii_case`], but works for anything that's `Display` without
    /// allocations
    fn repr_eq_ignore_ascii_case(&self, other: &str) -> bool {
        /// Writer that only succeeds if all of the input is a prefix of the contained string.
        struct X<'src>(&'src str);

        impl Write for X<'_> {
            fn write_str(&mut self, chunk: &str) -> std::fmt::Result {
                if !self
                    .0
                    .get(..chunk.len())
                    .is_some_and(|x| x.eq_ignore_ascii_case(chunk))
                {
                    return Err(std::fmt::Error);
                }
                self.0 = self.0.split_at(chunk.len()).1;
                Ok(())
            }
        }

        // The `is_ok_and` call ensures that there's nothing left over, ensuring
        // `s1.to_string().eq_ignore_ascii_case(s2)`
        // without ever allocating `s1`
        let mut writer = X(other);
        write!(writer, "{self}").is_ok_and(|_| writer.0.is_empty())
    }

    /// Equivalent of `s1.to_string() == s2` but without allocations
    fn repr_eq(&self, other: &str) -> bool {
        /// Writer that only succeeds if all of the input is a prefix of the contained string.
        struct X<'src>(&'src str);

        impl Write for X<'_> {
            fn write_str(&mut self, chunk: &str) -> std::fmt::Result {
                self.0
                    .strip_prefix(chunk)
                    .map(|rest| self.0 = rest)
                    .ok_or(std::fmt::Error)
            }
        }

        // The `is_ok_and` call ensures that there's nothing left over, ensuring `s1.to_string() == s2`
        // without ever allocating `s1`
        let mut writer = X(other);
        write!(writer, "{self}").is_ok_and(|_| writer.0.is_empty())
    }

    /// Equivalent of [`str::starts_with`], but works for anything that's `Display` without allocations
    fn starts_with(&self, prefix: &str) -> bool {
        /// Writer that only succeeds if all of the input is a prefix of the contained string.
        struct X<'src>(&'src str);

        impl Write for X<'_> {
            fn write_str(&mut self, s: &str) -> std::fmt::Result {
                match self.0.strip_prefix(s) {
                    Some(rest) => self.0 = rest,
                    None if self.0.len() < s.len() => {
                        s.strip_prefix(self.0).ok_or(std::fmt::Error)?;
                        self.0 = "";
                    }
                    None => return Err(std::fmt::Error),
                }

                Ok(())
            }
        }

        let mut writer = X(prefix);
        write!(writer, "{self}").is_ok()
    }

    /// Returns `true` if `s` only displays ASCII chars & doesn't start with a capital letter
    fn is_non_capitalized_ascii(&self) -> bool {
        /// Writer that succeeds only if the input is non-capitalised ASCII
        struct X {
            empty: bool,
        }

        impl Write for X {
            fn write_str(&mut self, mut s: &str) -> std::fmt::Result {
                if self.empty {
                    self.empty = s.is_empty();
                    let mut iter = s.chars();
                    if iter.next().is_some_and(|c| c.is_ascii_uppercase()) {
                        return Err(std::fmt::Error);
                    }
                    s = iter.as_str();
                }

                s.is_ascii().then_some(()).ok_or(std::fmt::Error)
            }
        }

        let mut writer = X { empty: true };
        write!(writer, "{self}").is_ok_and(|_| !writer.empty)
    }
}

impl<T: Display> DisplayExt for T {}

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
