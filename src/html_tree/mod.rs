use crate::{is_ide_completion, PeekValue};
use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::buffer::Cursor;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{braced, token, Token};

mod html_block;
mod html_component;
mod html_dashed_name;
mod html_element;
mod html_for;
mod html_if;
mod html_iterable;
mod html_let;
mod html_list;
mod html_match;
mod html_node;
mod tag;

use html_block::HtmlBlock;
use html_component::HtmlComponent;
pub use html_dashed_name::HtmlDashedName;
use html_element::HtmlElement;
use html_if::HtmlIf;
use html_iterable::HtmlIterable;
use html_list::HtmlList;
use html_node::HtmlNode;
use tag::TagTokens;

use self::html_block::BlockContent;
use self::html_for::HtmlFor;
use self::html_let::HtmlLet;
use self::html_match::HtmlMatch;

struct TokenIter<'cursor>(Cursor<'cursor>);

impl Iterator for TokenIter<'_> {
    type Item = TokenTree;

    fn next(&mut self) -> Option<Self::Item> {
        let (token, new_cursor) = self.0.token_tree()?;
        self.0 = new_cursor;
        Some(token)
    }
}

pub enum HtmlType {
    Block,
    Component,
    List,
    Element,
    If,
    For,
    Match,
    Empty,
}

pub enum HtmlTree {
    Block(Box<HtmlBlock>),
    Component(Box<HtmlComponent>),
    List(Box<HtmlList>),
    Element(Box<HtmlElement>),
    If(Box<HtmlIf>),
    For(Box<HtmlFor>),
    Match(Box<HtmlMatch>),
    Empty,
}

impl Parse for HtmlTree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let html_type = Self::peek_html_type(input)
            .ok_or_else(|| input.error("expected a valid html element"))?;
        let html_tree = match html_type {
            HtmlType::Empty => HtmlTree::Empty,
            HtmlType::Component => HtmlTree::Component(input.parse()?),
            HtmlType::Element => HtmlTree::Element(input.parse()?),
            HtmlType::Block => HtmlTree::Block(input.parse()?),
            HtmlType::List => HtmlTree::List(input.parse()?),
            HtmlType::If => HtmlTree::If(input.parse()?),
            HtmlType::For => HtmlTree::For(input.parse()?),
            HtmlType::Match => HtmlTree::Match(input.parse()?),
        };
        Ok(html_tree)
    }
}

impl HtmlTree {
    /// Determine the [`HtmlType`] before actually parsing it.
    /// Even though this method accepts a [`ParseStream`], it is forked and the original stream is
    /// not modified. Once a certain `HtmlType` can be deduced for certain, the function eagerly
    /// returns with the appropriate type. If invalid html tag, returns `None`.
    fn peek_html_type(input: ParseStream) -> Option<HtmlType> {
        let input = input.fork(); // do not modify original ParseStream
        let cursor = input.cursor();

        if input.is_empty() {
            Some(HtmlType::Empty)
        } else if cursor.group(proc_macro2::Delimiter::Brace).is_some() {
            Some(HtmlType::Block)
        } else if HtmlIf::peek(cursor).is_some() {
            Some(HtmlType::If)
        } else if HtmlFor::peek(cursor).is_some() {
            Some(HtmlType::For)
        } else if HtmlMatch::peek(cursor).is_some() {
            Some(HtmlType::Match)
        } else if input.peek(Token![<]) {
            let _lt: Token![<] = input.parse().ok()?;

            // eat '/' character for unmatched closing tag
            let _slash: Option<Token![/]> = input.parse().ok();

            if input.peek(Token![>]) {
                Some(HtmlType::List)
            } else if input.peek(Token![@]) {
                Some(HtmlType::Element) // dynamic element
            } else if input.peek(Token![::]) {
                Some(HtmlType::Component)
            } else if input.peek(Ident::peek_any) {
                let ident = Ident::parse_any(&input).ok()?;
                let ident_str = ident.to_string();

                if input.peek(Token![=]) || (input.peek(Token![?]) && input.peek2(Token![=])) {
                    Some(HtmlType::List)
                } else if ident_str.chars().next().unwrap().is_ascii_uppercase()
                    || input.peek(Token![::])
                    || is_ide_completion() && ident_str.chars().any(|c| c.is_ascii_uppercase())
                {
                    Some(HtmlType::Component)
                } else {
                    Some(HtmlType::Element)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl ToTokens for HtmlTree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            HtmlTree::Empty => tokens.extend(quote! {
                ::yew::virtual_dom::VNode::VList(
                    ::std::rc::Rc::new(
                        ::yew::virtual_dom::VList::new()
                    )
                )
            }),
            HtmlTree::Component(comp) => comp.to_tokens(tokens),
            HtmlTree::Element(tag) => tag.to_tokens(tokens),
            HtmlTree::List(list) => list.to_tokens(tokens),
            HtmlTree::Block(block) => block.to_tokens(tokens),
            HtmlTree::If(block) => block.to_tokens(tokens),
            HtmlTree::For(block) => block.to_tokens(tokens),
            HtmlTree::Match(block) => block.to_tokens(tokens),
        }
    }
}

pub enum HtmlRoot {
    Empty,
    Tree(HtmlTree),
    Trees(Vec<HtmlTree>),
    Iterable(Box<HtmlIterable>),
    Node(Box<HtmlNode>),
}

impl Parse for HtmlRoot {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        /// Assumes that `cursor` is pointing at a `for` token
        fn for_loop_like(cursor: Cursor<'_>) -> bool {
            let Some((_, cursor)) = cursor.token_tree() else {
                return false;
            };
            // The `take_while` call makes sure that e.g. `html!(for for i in 0 .. 10 {})` is
            // classified correctly
            TokenIter(cursor)
                .take_while(|t| !matches!(t, TokenTree::Ident(i) if i == "for"))
                .any(|t| matches!(t, TokenTree::Ident(i) if i == "in"))
        }

        let first = match HtmlTree::peek_html_type(input) {
            Some(HtmlType::For) => {
                if for_loop_like(input.cursor()) {
                    HtmlTree::For(input.parse()?)
                } else {
                    return Ok(Self::Iterable(input.parse()?));
                }
            }
            Some(HtmlType::Match) => HtmlTree::Match(input.parse()?),
            Some(HtmlType::Block) => HtmlTree::Block(input.parse()?),
            Some(HtmlType::Component) => HtmlTree::Component(input.parse()?),
            Some(HtmlType::List) => HtmlTree::List(input.parse()?),
            Some(HtmlType::Element) => HtmlTree::Element(input.parse()?),
            Some(HtmlType::If) => HtmlTree::If(input.parse()?),
            Some(HtmlType::Empty) => return Ok(Self::Empty),
            None => return Ok(Self::Node(input.parse()?)),
        };

        if input.is_empty() {
            return Ok(Self::Tree(first));
        }

        let mut nodes = vec![first];
        while !input.is_empty() {
            nodes.push(input.parse()?);
        }

        Ok(Self::Trees(nodes))
    }
}

impl ToTokens for HtmlRoot {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Empty => HtmlTree::Empty.to_tokens(tokens),
            Self::Tree(tree) => tree.to_tokens(tokens),
            Self::Trees(children) => children_to_vnode_tokens(&[], children, tokens),
            Self::Node(node) => node.to_tokens(tokens),
            Self::Iterable(iterable) => iterable.to_tokens(tokens),
        }
    }
}

/// Same as HtmlRoot but always returns a VNode.
pub struct AsVNode<T>(pub T);
impl<T: Parse> Parse for AsVNode<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse().map(Self)
    }
}

impl<T: ToTokens> ToTokens for AsVNode<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = &self.0;
        tokens.extend(
            quote_spanned! {inner.span().resolved_at(Span::mixed_site())=> {
                #[allow(clippy::useless_conversion)]
                <::yew::virtual_dom::VNode as ::std::convert::From<_>>::from(#inner)
            }},
        );
    }
}

/// This trait represents a type that can be unfolded into multiple html nodes.
pub trait ToNodeIterator {
    /// Generate a token stream which produces a value that implements IntoIterator<Item=T> where T
    /// is inferred by the compiler. The easiest way to achieve this is to call `.into()` on
    /// each element. If the resulting iterator only ever yields a single item this function
    /// should return None instead.
    fn to_node_iterator_stream(&self) -> Option<TokenStream>;
}

impl ToNodeIterator for HtmlTree {
    fn to_node_iterator_stream(&self) -> Option<TokenStream> {
        match self {
            HtmlTree::Block(block) => block.to_node_iterator_stream(),
            // everything else is just a single node.
            _ => None,
        }
    }
}

pub struct HtmlChildrenTree {
    pub bindings: Vec<HtmlLet>,
    pub children: Vec<HtmlTree>,
}

// Check if each child represents a single node.
// This is the case when no expressions are used.
fn only_single_node_children(children: &[impl ToNodeIterator]) -> bool {
    children
        .iter()
        .map(ToNodeIterator::to_node_iterator_stream)
        .all(|s| s.is_none())
}

pub fn to_build_vec_tokens(
    bindings: &[HtmlLet],
    children: &[HtmlTree],
    tokens: &mut TokenStream,
) {
    if only_single_node_children(children) {
        // optimize for the common case where all children are single nodes (only using literal
        // html).
        let children_into = children
            .iter()
            .map(|child| quote_spanned! {child.span()=> ::std::convert::Into::into(#child) });
        tokens.extend(if bindings.is_empty() {
            quote! { ::std::vec![#(#children_into),*] }
        } else {
            quote! {
                {
                    #(#bindings)*
                    ::std::vec![#(#children_into),*]
                }
            }
        });
        return;
    }

    let vec_ident = Ident::new("__yew_v", Span::mixed_site());
    let add_children_streams = children.iter().map(|child| {
        if let Some(node_iterator_stream) = child.to_node_iterator_stream() {
            quote! {
                ::std::iter::Extend::extend(&mut #vec_ident, #node_iterator_stream);
            }
        } else {
            quote_spanned! {child.span()=>
                #vec_ident.push(::std::convert::Into::into(#child));
            }
        }
    });

    tokens.extend(quote! {
        {
            #(#bindings;)*
            let mut #vec_ident = ::std::vec::Vec::new();
            #(#add_children_streams)*
            #vec_ident
        }
    });
}

pub fn children_to_vnode_tokens(
    bindings: &[HtmlLet],
    children: &[HtmlTree],
    tokens: &mut TokenStream,
) {
    let res = match children[..] {
        [] => quote! {::std::default::Default::default() },
        [HtmlTree::Component(ref children)] => {
            quote! { ::yew::html::IntoPropValue::<::yew::virtual_dom::VNode>::into_prop_value(#children) }
        }
        [HtmlTree::Element(ref children)] => {
            quote! { ::yew::html::IntoPropValue::<::yew::virtual_dom::VNode>::into_prop_value(#children) }
        }
        [HtmlTree::Block(ref m)] => {
            // We only want to process `{vnode}` and not `{for vnodes}`.
            // This should be converted into a if let guard once https://github.com/rust-lang/rust/issues/51114 is stable.
            // Or further nested once deref pattern (https://github.com/rust-lang/rust/issues/87121) is stable.
            if let HtmlBlock {
                content: BlockContent::Node(children),
                ..
            } = m.as_ref()
            {
                quote! { ::yew::html::IntoPropValue::<::yew::virtual_dom::VNode>::into_prop_value(#children) }
            } else {
                let mut children_vec = TokenStream::new();
                to_build_vec_tokens(bindings, children, &mut children_vec);
                tokens.extend(quote! {
                    ::yew::html::IntoPropValue::<::yew::virtual_dom::VNode>::into_prop_value(
                        ::yew::html::ChildrenRenderer::new(#children_vec)
                    )
                });
                return;
            }
        }
        _ => {
            let mut children_vec = TokenStream::new();
            to_build_vec_tokens(bindings, children, &mut children_vec);
            tokens.extend(quote! {
                ::yew::html::IntoPropValue::<::yew::virtual_dom::VNode>::into_prop_value(
                    ::yew::html::ChildrenRenderer::new(#children_vec)
                )
            });
            return;
        }
    };

    tokens.extend(if bindings.is_empty() {
        res
    } else {
        quote! {
            {
                #(#bindings;)*
                #res
            }
        }
    })
}

impl HtmlChildrenTree {
    pub fn new() -> Self {
        Self {
            children: vec![],
            bindings: vec![],
        }
    }

    pub fn parse_child(&mut self, input: ParseStream) -> syn::Result<()> {
        if input.peek(Token![let]) {
            if !self.children.is_empty() {
                return Err(input.error("`let` bindings must come before any children"));
            }
            self.bindings.push(input.parse()?)
        } else {
            self.children.push(input.parse()?)
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    fn parse_delimited(input: ParseStream) -> syn::Result<Self> {
        let mut children = HtmlChildrenTree::new();

        while !input.is_empty() {
            children.parse_child(input)?;
        }

        Ok(children)
    }

    pub fn to_children_renderer_tokens(&self) -> Option<TokenStream> {
        let Self { bindings, children } = self;

        let res = match children[..] {
            [] => None,
            [HtmlTree::Component(ref children)] => Some(quote! { #children }),
            [HtmlTree::Element(ref children)] => Some(quote! { #children }),
            [HtmlTree::Block(ref m)] => {
                // We only want to process `{vnode}` and not `{for vnodes}`.
                // This should be converted into a if let guard once https://github.com/rust-lang/rust/issues/51114 is stable.
                // Or further nested once deref pattern (https://github.com/rust-lang/rust/issues/87121) is stable.
                if let HtmlBlock {
                    content: BlockContent::Node(children),
                    ..
                } = m.as_ref()
                {
                    Some(quote! { #children })
                } else {
                    return Some(quote! { ::yew::html::ChildrenRenderer::new(#self) });
                }
            }
            _ => return Some(quote! { ::yew::html::ChildrenRenderer::new(#self) }),
        };
        if bindings.is_empty() {
            res
        } else {
            Some(quote! {
                {
                    #(#bindings;)*
                    #res
                }
            })
        }
    }

    pub fn to_vnode_tokens(&self) -> TokenStream {
        let mut res = TokenStream::new();
        children_to_vnode_tokens(&self.bindings, &self.children, &mut res);
        res
    }

    pub fn size_hint(&self) -> Option<usize> {
        only_single_node_children(&self.children)
            .then_some(self.children.len())
    }
}

impl ToTokens for HtmlChildrenTree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        to_build_vec_tokens(&self.bindings, &self.children, tokens);
    }
}

pub struct HtmlRootBraced {
    brace: token::Brace,
    children: HtmlChildrenTree,
}

impl PeekValue<()> for HtmlRootBraced {
    fn peek(cursor: Cursor) -> Option<()> {
        cursor.group(Delimiter::Brace).map(|_| ())
    }
}

impl Parse for HtmlRootBraced {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let brace = braced!(content in input);
        let children = HtmlChildrenTree::parse_delimited(&content)?;

        Ok(HtmlRootBraced { brace, children })
    }
}

impl ToTokens for HtmlRootBraced {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { brace, children } = self;

        tokens.extend(quote_spanned! {brace.span.span()=>
            {
                ::yew::virtual_dom::VNode::VList(
                    ::std::rc::Rc::new(
                        ::yew::virtual_dom::VList::with_children(
                            #children,
                            ::std::option::Option::None
                        )
                    )
                )
            }
        });
    }
}
