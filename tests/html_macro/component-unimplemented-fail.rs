struct Unimplemented;

fn compile_fail() {
    ::yew_html_ext::html! { <Unimplemented /> };
}

fn main() {}
