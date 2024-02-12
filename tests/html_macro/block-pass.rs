#![no_implicit_prelude]

// Shadow primitives
#[allow(non_camel_case_types)]
pub struct bool;
#[allow(non_camel_case_types)]
pub struct char;
#[allow(non_camel_case_types)]
pub struct f32;
#[allow(non_camel_case_types)]
pub struct f64;
#[allow(non_camel_case_types)]
pub struct i128;
#[allow(non_camel_case_types)]
pub struct i16;
#[allow(non_camel_case_types)]
pub struct i32;
#[allow(non_camel_case_types)]
pub struct i64;
#[allow(non_camel_case_types)]
pub struct i8;
#[allow(non_camel_case_types)]
pub struct isize;
#[allow(non_camel_case_types)]
pub struct str;
#[allow(non_camel_case_types)]
pub struct u128;
#[allow(non_camel_case_types)]
pub struct u16;
#[allow(non_camel_case_types)]
pub struct u32;
#[allow(non_camel_case_types)]
pub struct u64;
#[allow(non_camel_case_types)]
pub struct u8;
#[allow(non_camel_case_types)]
pub struct usize;

use ::yew_html_ext::html;

fn main() {
    _ = html! { <>{ "Hi" }</> };
    _ = html! { <>{ ::std::format!("Hello") }</> };
    _ = html! { <>{ ::std::string::ToString::to_string("Hello") }</> };

    let msg = "Hello";
    _ = html! { <div>{ msg }</div> };

    let subview = html! { "subview!" };
    _ = html! { <div>{ subview }</div> };

    let subview = || html! { "subview!" };
    _ = html! { <div>{ subview() }</div> };

    _ = html! {
        <ul>
            { for ::std::iter::Iterator::map(0..3, |num| { html! { <span>{ num }</span> }}) }
        </ul>
    };

    let item = |num| html! { <li>{ ::std::format!("item {}!", num) }</li> };
    _ = html! {
        <ul>
            { for ::std::iter::Iterator::map(0..3, item) }
        </ul>
    };
}
