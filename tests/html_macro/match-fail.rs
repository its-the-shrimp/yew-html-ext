fn main() {
    _ = ::yew_html_ext::html! {
        match ::std::option::Option::Some(3u32) {
            ::std::option::Option::Some(_) => <div/>,
            ::std::option::Option::None => {},
        }
    }
}
