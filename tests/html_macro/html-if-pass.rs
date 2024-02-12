#![no_implicit_prelude]

use ::yew_html_ext::html;

fn compile_pass_lit() {
    _ = html! { if true {} };
    _ = html! { if true { <div/> } };
    _ = html! { if true { <div/><div/> } };
    _ = html! { if true { <><div/><div/></> } };
    _ = html! { if true { { html! {} } } };
    _ = html! { if true { { { let _x = 42; html! {} } } } };
    _ = html! { if true {} else {} };
    _ = html! { if true {} else if true {} };
    _ = html! { if true {} else if true {} else {} };
    _ = html! { if let ::std::option::Option::Some(text) = ::std::option::Option::Some("text") { <span>{ text }</span> } };
    _ = html! { <><div/>if true {}<div/></> };
    _ = html! { <div>if true {}</div> };
}

fn compile_pass_expr() {
    let condition = true;

    _ = html! { if condition {} };
    _ = html! { if condition { <div/> } };
    _ = html! { if condition { <div/><div/> } };
    _ = html! { if condition { <><div/><div/></> } };
    _ = html! { if condition { { html! {} } } };
    _ = html! { if condition { { { let _x = 42; html! {} } } } };
    _ = html! { if condition {} else {} };
    _ = html! { if condition {} else if condition {} };
    _ = html! { if condition {} else if condition {} else {} };
    _ = html! { if let ::std::option::Option::Some(text) = ::std::option::Option::Some("text") { <span>{ text }</span> } };
    _ = html! { <><div/>if condition {}<div/></> };
    _ = html! { <div>if condition {}</div> };
}

fn main() {}
