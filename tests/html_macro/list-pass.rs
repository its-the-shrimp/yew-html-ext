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
    _ = html! {};
    _ = html! { <></> };
    _ = html! {
        <>
            <></>
            <></>
        </>
    };
    _ = html! {
        <key={::std::string::ToString::to_string("key")}>
        </>
    };

    let children = ::std::vec![
        html! { <span>{ "Hello" }</span> },
        html! { <span>{ "World" }</span> },
    ];
    _ = html! { <>{ children }</> };

    // Testing the implicit top-level list
    _ = html! {
        <div />
        <div />
    };
}
