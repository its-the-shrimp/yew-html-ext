use ::yew_html_ext::html;

fn compile_fail() {
    // missing closing tag
    html! { <> };
    html! { <><> };
    html! { <><></> };

    // missing starting tag
    html! { </> };
    html! { </></> };

    // invalid child content
    html! { <>invalid</> };
    // no key value
    html! { <key=></> };
    // wrong closing tag
    html! { <key="key".to_string()></key> };

    // multiple keys
    html! { <key="first key" key="second key" /> };
    // invalid prop
    html! { <some_attr="test"></> };
}

fn main() {}
