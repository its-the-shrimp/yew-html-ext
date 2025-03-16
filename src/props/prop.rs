use std::convert::TryFrom;
use std::io::Write;
use std::ops::{Deref, DerefMut};

use proc_macro2::{Spacing, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::parse::{ParseBuffer, ParseStream};
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{
    braced, Attribute, Block, Expr, ExprBlock, ExprLit, ExprMacro, ExprPath, ExprRange, Lit,
    LitStr, Meta, Stmt, Token,
};

use crate::html_tree::HtmlDashedName;
use crate::stringify::Stringify;

#[derive(Copy, Clone)]
pub enum PropDirective {
    ApplyAsProperty(Token![~]),
}

pub struct Prop {
    pub cfg: Option<TokenStream>,
    pub directive: Option<PropDirective>,
    pub label: HtmlDashedName,
    /// Punctuation between `label` and `value`.
    pub value: Expr,
}

impl Prop {
    pub fn parse(input: ParseStream, element: Option<&HtmlDashedName>) -> syn::Result<Self> {
        let cfg = Attribute::parse_outer(input)?
            .into_iter()
            .map(|attr| match attr.meta {
                Meta::List(list) if list.path.is_ident("cfg") => Ok(list.tokens),
                _ => Err(syn::Error::new_spanned(
                    attr,
                    "only the `#[cfg]` attribute is permitted on props",
                )),
            })
            .reduce(|acc, i| {
                let (acc, i) = (acc?, i?);
                Ok(quote! { all(#acc, #i) })
            })
            .transpose()?;

        let directive = input
            .parse::<Token![~]>()
            .map(PropDirective::ApplyAsProperty)
            .ok();
        if input.peek(Brace) {
            Self::parse_shorthand_prop_assignment(input, directive, cfg)
        } else {
            Self::parse_prop_assignment(input, directive, cfg, element)
        }
    }
}

/// Helpers for parsing props
impl Prop {
    /// Parse a prop using the shorthand syntax `{value}`, short for `value={value}`
    /// This only allows for labels with no hyphens, as it would otherwise create
    /// an ambiguity in the syntax
    fn parse_shorthand_prop_assignment(
        input: ParseStream,
        directive: Option<PropDirective>,
        cfg: Option<TokenStream>,
    ) -> syn::Result<Self> {
        let value;
        let _brace = braced!(value in input);
        let expr = value.parse::<Expr>()?;
        let label = if let Expr::Path(ExprPath {
            ref attrs,
            qself: None,
            ref path,
        }) = expr
        {
            if let (Some(ident), true) = (path.get_ident(), attrs.is_empty()) {
                Ok(HtmlDashedName::from(ident.clone()))
            } else {
                Err(syn::Error::new_spanned(
                    path,
                    "only simple identifiers are allowed in the shorthand property syntax",
                ))
            }
        } else {
            return Err(syn::Error::new_spanned(
                expr,
                "missing label for property value. If trying to use the shorthand property \
                 syntax, only identifiers may be used",
            ));
        }?;

        Ok(Self {
            label,
            value: expr,
            directive,
            cfg,
        })
    }

    /// Parse a prop of the form `label={value}`
    fn parse_prop_assignment(
        input: ParseStream,
        directive: Option<PropDirective>,
        cfg: Option<TokenStream>,
        element: Option<&HtmlDashedName>,
    ) -> syn::Result<Self> {
        let label = input.parse::<HtmlDashedName>()?;
        let equals = input.parse::<Token![=]>().map_err(|_| {
            syn::Error::new_spanned(
                &label,
                format!(
                    "`{label}` doesn't have a value. (hint: set the value to `true` or `false` \
                     for boolean attributes)"
                ),
            )
        })?;
        if input.is_empty() {
            return Err(syn::Error::new_spanned(
                equals,
                "expected an expression following this equals sign",
            ));
        }

        let is_inline_css_attr = element.is_some_and(|e| is_inline_css_attr(e, &label));
        let value = parse_prop_value(input, is_inline_css_attr)?;
        Ok(Self {
            label,
            value,
            directive,
            cfg,
        })
    }
}

pub fn is_inline_css_attr(element: &HtmlDashedName, attr: &HtmlDashedName) -> bool {
    let mut buf = [0u8; 16];
    if write!(&mut buf[..], "{}", element.name).is_err() {
        return false; // If this reached, the name's too long & wouldn't pass the check anyway
    };
    let element = str::from_utf8(&buf)
        .expect("UTF-8 element name")
        .trim_end_matches('\0');

    match element {
        "a" | "abbr" | "article" | "aside" | "audio" | "b" | "blockquote" | "br" | "button"
        | "canvas" | "caption" | "cite" | "code" | "col" | "colgroup" | "details" | "div"
        | "dl" | "dt" | "dd" | "em" | "figcaption" | "figure" | "fieldset" | "footer" | "form"
        | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "header" | "hr" | "i" | "iframe" | "img"
        | "input" | "label" | "legend" | "li" | "main" | "mark" | "meter" | "nav" | "ol"
        | "option" | "p" | "pre" | "progress" | "section" | "select" | "small" | "span"
        | "strong" | "sub" | "summary" | "sup" | "table" | "tbody" | "td" | "textarea"
        | "tfoot" | "th" | "thead" | "time" | "tr" | "u" | "ul" | "video" => attr.name == "style",

        _ => false,
    }
}

fn parse_prop_value(input: &ParseBuffer, is_inline_css_attr: bool) -> syn::Result<Expr> {
    if input.peek(Brace) {
        strip_braces(input.parse()?)
    } else {
        let expr = if let Some(ExprRange {
            start: Some(start), ..
        }) = range_expression_peek(input)
        {
            // If a range expression is seen, treat the left-side expression as the value
            // and leave the right-side expression to be parsed as a base expression
            advance_until_next_dot2(input)?;
            *start
        } else {
            input.parse()?
        };

        match &expr {
            Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) if is_inline_css_attr => Ok(Expr::Lit(ExprLit {
                attrs: vec![],
                lit: Lit::Str(LitStr::new(&minify_css(s.value()), s.span())),
            })),
            Expr::Lit(_) => Ok(expr),
            _ => Err(syn::Error::new_spanned(
                &expr,
                "the property value must be either a literal or enclosed in braces. Consider \
                 adding braces around your expression.",
            )),
        }
    }
}

fn strip_braces(block: ExprBlock) -> syn::Result<Expr> {
    match block {
        ExprBlock {
            block: Block { mut stmts, .. },
            ..
        } if stmts.len() == 1 => {
            let stmt = stmts.remove(0);
            match stmt {
                Stmt::Expr(expr, None) => Ok(expr),
                Stmt::Macro(mac) => Ok(Expr::Macro(ExprMacro {
                    attrs: vec![],
                    mac: mac.mac,
                })),
                // See issue #2267, we want to parse macro invocations as expressions
                Stmt::Item(syn::Item::Macro(mac))
                    if mac.ident.is_none() && mac.semi_token.is_none() =>
                {
                    Ok(Expr::Macro(syn::ExprMacro {
                        attrs: mac.attrs,
                        mac: mac.mac,
                    }))
                }
                Stmt::Expr(_, Some(semi)) => Err(syn::Error::new_spanned(
                    semi,
                    "only an expression may be assigned as a property. Consider removing this \
                     semicolon",
                )),
                _ => Err(syn::Error::new_spanned(
                    stmt,
                    "only an expression may be assigned as a property",
                )),
            }
        }
        block => Ok(Expr::Block(block)),
    }
}

// Without advancing cursor, returns the range expression at the current cursor position if any
fn range_expression_peek(input: &ParseBuffer) -> Option<ExprRange> {
    match input.fork().parse::<Expr>().ok()? {
        Expr::Range(range) => Some(range),
        _ => None,
    }
}

fn advance_until_next_dot2(input: &ParseBuffer) -> syn::Result<()> {
    input.step(|cursor| {
        let mut rest = *cursor;
        let mut first_dot = None;
        while let Some((tt, next)) = rest.token_tree() {
            match &tt {
                TokenTree::Punct(punct) if punct.as_char() == '.' => {
                    if let Some(first_dot) = first_dot {
                        return Ok(((), first_dot));
                    } else {
                        // Only consider dot as potential first if there is no spacing after it
                        first_dot = if punct.spacing() == Spacing::Joint {
                            Some(rest)
                        } else {
                            None
                        };
                    }
                }
                _ => {
                    first_dot = None;
                }
            }
            rest = next;
        }
        Err(cursor.error("no `..` found in expression"))
    })
}

/// - Strips all whitespace after a unescaped [semi]colon that's not inside quotes
/// - Removes the final semicolon
pub fn minify_css(mut s: String) -> String {
    let mut stripping = false;
    let mut escaped = false;
    let mut in_quotes = false;

    s.retain(|c| {
        if stripping {
            if c.is_ascii_whitespace() {
                return false;
            }
            stripping = false;
            in_quotes = c == '"';
            return true;
        }

        match c {
            '"' if !escaped => in_quotes = !in_quotes,
            ':' | ';' if !in_quotes => stripping = true,
            _ => escaped = c == '\\',
        }

        true
    });

    if s.ends_with(';') {
        s.pop();
    }

    s
}

/// List of props sorted in alphabetical order*.
///
/// \*The "children" prop always comes last to match the behaviour of the `Properties` derive macro.
///
/// The list may contain multiple props with the same label.
/// Use `check_no_duplicates` to ensure that there are no duplicates.
pub struct PropList(Vec<Prop>);
impl PropList {
    /// Create a new `SortedPropList` from a vector of props.
    /// The given `props` doesn't need to be sorted.
    pub fn new(props: Vec<Prop>) -> Self {
        Self(props)
    }

    fn position(&self, key: &str) -> Option<usize> {
        self.0.iter().position(|it| it.label.to_string() == key)
    }

    /// Get the first prop with the given key.
    pub fn get_by_label(&self, key: &str) -> Option<&Prop> {
        self.0.iter().find(|it| it.label.to_string() == key)
    }

    /// Pop the first prop with the given key.
    pub fn pop(&mut self, key: &str) -> Option<Prop> {
        self.position(key).map(|i| self.0.remove(i))
    }

    /// Pop the prop with the given key and error if there are multiple ones.
    pub fn pop_unique(&mut self, key: &str) -> syn::Result<Option<Prop>> {
        let prop = self.pop(key);
        if prop.is_some() {
            if let Some(other_prop) = self.get_by_label(key) {
                return Err(syn::Error::new_spanned(
                    &other_prop.label,
                    format!("`{key}` can only be specified once"),
                ));
            }
        }

        Ok(prop)
    }

    /// Turn the props into a vector of `Prop`.
    pub fn into_vec(self) -> Vec<Prop> {
        self.0
    }

    /// Iterate over all duplicate props in order of appearance.
    fn iter_duplicates(&self) -> impl Iterator<Item = &Prop> {
        self.0.windows(2).filter_map(|pair| {
            let (a, b) = (&pair[0], &pair[1]);

            if a.label == b.label {
                Some(b)
            } else {
                None
            }
        })
    }

    /// Remove and return all props for which `filter` returns `true`.
    pub fn drain_filter(&mut self, filter: impl FnMut(&Prop) -> bool) -> Self {
        let (drained, others) = self.0.drain(..).partition(filter);
        self.0 = others;
        Self(drained)
    }

    /// Run the given function for all props and aggregate the errors.
    /// If there's at least one error, the result will be `Result::Err`.
    pub fn check_all(&self, f: impl FnMut(&Prop) -> syn::Result<()>) -> syn::Result<()> {
        crate::join_errors(self.0.iter().map(f).filter_map(Result::err))
    }

    /// Return an error for all duplicate props.
    pub fn check_no_duplicates(&self) -> syn::Result<()> {
        crate::join_errors(self.iter_duplicates().map(|prop| {
            syn::Error::new_spanned(
                &prop.label,
                format!(
                    "`{}` can only be specified once but is given here again",
                    prop.label
                ),
            )
        }))
    }
}

impl PropList {
    pub fn parse(input: ParseStream, element: Option<&HtmlDashedName>) -> syn::Result<Self> {
        let mut props: Vec<Prop> = Vec::new();
        // Stop parsing props if a base expression preceded by `..` is reached
        while !input.is_empty() && !input.peek(Token![..]) {
            props.push(Prop::parse(input, element)?);
        }

        Ok(Self::new(props))
    }
}

impl Deref for PropList {
    type Target = [Prop];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct SpecialProps {
    pub node_ref: Option<Prop>,
    pub key: Option<Prop>,
}
impl SpecialProps {
    const KEY_LABEL: &'static str = "key";
    const REF_LABEL: &'static str = "ref";

    fn pop_from(props: &mut PropList) -> syn::Result<Self> {
        let node_ref = props.pop_unique(Self::REF_LABEL)?;
        let key = props.pop_unique(Self::KEY_LABEL)?;
        Ok(Self { node_ref, key })
    }

    pub fn wrap_node_ref_attr(&self) -> TokenStream {
        self.node_ref
            .as_ref()
            .map(|attr| {
                let value = &attr.value;
                let cfg1 = attr.cfg.iter();
                let cfg2 = attr.cfg.iter();
                quote_spanned! {value.span().resolved_at(Span::call_site())=> {
                    #(#[cfg(#cfg1)])*
                    let x = ::yew::html::IntoPropValue::<::yew::html::NodeRef>::into_prop_value(#value);
                    #(
                        #[cfg(not(#cfg2))]
                        let x = <::yew::html::NodeRef as ::std::default::Default>::default();
                    )*
                    x
                }}
            })
            .unwrap_or(quote! { ::std::default::Default::default() })
    }

    pub fn wrap_key_attr(&self) -> TokenStream {
        self.key
            .as_ref()
            .map(|attr| {
                let value = attr.value.optimize_literals();
                let cfg1 = attr.cfg.iter();
                let cfg2 = attr.cfg.iter();
                quote_spanned! {value.span().resolved_at(Span::call_site())=> {
                    #(#[cfg(#cfg1)])*
                    let x = ::std::option::Option::Some(
                        ::std::convert::Into::<::yew::virtual_dom::Key>::into(#value)
                    );
                    #(
                        #[cfg(not(#cfg2))]
                        let x = ::std::option::Option::None;
                    )*
                    x
                }}
            })
            .unwrap_or(quote! { ::std::option::Option::None })
    }
}

pub struct Props {
    pub special: SpecialProps,
    pub prop_list: PropList,
}

impl Props {
    pub fn parse(input: ParseStream, element: Option<&HtmlDashedName>) -> syn::Result<Self> {
        Self::try_from(PropList::parse(input, element)?)
    }
}

impl Deref for Props {
    type Target = PropList;

    fn deref(&self) -> &Self::Target {
        &self.prop_list
    }
}

impl DerefMut for Props {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.prop_list
    }
}

impl TryFrom<PropList> for Props {
    type Error = syn::Error;

    fn try_from(mut prop_list: PropList) -> Result<Self, Self::Error> {
        let special = SpecialProps::pop_from(&mut prop_list)?;
        Ok(Self { special, prop_list })
    }
}
