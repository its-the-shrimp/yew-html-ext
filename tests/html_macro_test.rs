use yew_html_ext::{html, html_nested};
use yew::{function_component, Html, Properties};

#[allow(dead_code)]
//#[rustversion::attr(stable(1.67), test)]
#[test]
fn html_macro() {
    let t = trybuild::TestCases::new();

    t.pass("tests/html_macro/*-pass.rs");
    t.compile_fail("tests/html_macro/*-fail.rs");
}

#[test]
#[should_panic(
    expected = "a dynamic tag tried to create a `<br>` tag with children. `<br>` is a void \
                element which can't have any children."
)]
fn dynamic_tags_catch_void_elements() {
    let _ = html! {
        <@{"br"}>
            <span>{ "No children allowed" }</span>
        </@>
    };
}

#[test]
#[should_panic(expected = "a dynamic tag returned a tag name containing non ASCII characters: `❤`")]
fn dynamic_tags_catch_non_ascii() {
    let _ = html! { <@{"❤"} /> };
}

/// test that compilation on html elements pass
/// fixes: https://github.com/yewstack/yew/issues/2268
#[test]
fn html_nested_macro_on_html_element() {
    let _node = html_nested! { <div /> };
    let _node = html_nested! { <input /> };
}

#[test]
#[allow(unexpected_cfgs)]
fn props_are_cfged_out() {
    #[cfg(nothing)]
    compile_error!("defining cfg(nothing) breaks this test");

    #[derive(PartialEq, Properties)]
    struct FooProps {
        #[prop_or_default]
        x: i32,
    }
    
    #[function_component]
    fn Foo(_: &FooProps) -> Html {
        Html::default()
    }

    let x = html! { <div #[cfg(nothing)] id="id" #[cfg(nothing)] key="x" /> };
    let y = html! { <div /> };
    assert_eq!(x, y);

    let x = html! { <Foo #[cfg(nothing)] x=69 #[cfg(nothing)] key="x" /> };
    let y = html! { <Foo /> };
    assert_eq!(x, y);
}
