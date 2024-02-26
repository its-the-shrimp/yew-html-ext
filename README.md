# yew-html-ext
## Write HTML in Rust more concisely
`yew-html-ext` provides a drop-in replacement for Yew's [`html!`](https://docs.rs/yew/latest/yew/macro.html.html)
and [`html_nested!`](https://docs.rs/yew/latest/yew/macro.html_nested.html) macros with 
a number of syntactic extensions. Being a drop-in replacement, all one has to do to start benefitting from this library is:

1. Add it to the project's dependencies
```toml
[dependencies]
yew-html-ext = "0.1"
```
2. Replace uses/imports of `yew::html{_nested}` with `yew_html_ext::html{_nested}`

The provided macros facilitate an experimental ground for potential additions to Yew HTML proper,
which is why the base features are untouched, only new ones are added.
The specific syntax provided by the library is explained in [the docs](https://docs.rs/yew-html-ext/latest/yew_html_ext)

## Format this new fancy HTML
[`yew-fmt`](https://github.com/schvv31n/yew-fmt) has support for this extended syntax,
which is however opt-in and is to be enabled by adding the following line to `rustfmt.toml`:
```toml
yew.html_flavor = "Ext"
```
More on this [here](https://github.com/schvv31n/yew-fmt?tab=readme-ov-file#yewhtml_flavor)
