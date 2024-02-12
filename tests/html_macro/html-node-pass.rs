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

fn compile_pass() {
    _ = html! { "" };
    _ = html! { 'a' };
    _ = html! { "hello" };
    _ = html! { 42 };
    _ = html! { 1.234 };

    _ = html! { <span>{ "" }</span> };
    _ = html! { <span>{ 'a' }</span> };
    _ = html! { <span>{ "hello" }</span> };
    _ = html! { <span>{ 42 }</span> };
    _ = html! { <span>{ 1.234 }</span> };

    _ = html! { ::std::format!("Hello") };
    _ = html! { {<::std::string::String as ::std::convert::From<&::std::primitive::str>>::from("Hello") } };

    let msg = "Hello";
    _ = html! { msg };
}

fn main() {}
