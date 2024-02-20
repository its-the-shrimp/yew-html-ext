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
    let random = 4u8;
    _ = html! {
        if let ::std::option::Option::Some(x) = random.checked_add(1) {
            let x = ::std::format!("the random number is {x}");
            <p>{ ::std::clone::Clone::clone(&x) }</p>
            <p>{ "No, seriously, " }{ x }</p>
        }
    };

    _ = html! {
        <>
            let (min, max): (::std::primitive::u8, ::std::primitive::u8) = (1, 2);
            <h1>{ min }</h1>
            <h2>{ max }</h2>
        </>
    };

    _ = html! {
        <div>
            let ::std::option::Option::Some(x) = random.checked_mul(10) else { return };
            for i in x .. 10 {
                { i + x }
            }
        </div>
    }
}
