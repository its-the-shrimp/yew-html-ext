use ::yew_html_ext::html;

pub struct Generic<T> {
    marker: ::std::marker::PhantomData<T>,
}

impl<T> ::yew::html::Component for Generic<T>
where
    T: 'static,
{
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        unimplemented!()
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        unimplemented!()
    }
}

pub struct Generic2<T1, T2> {
    marker: ::std::marker::PhantomData<(T1, T2)>,
}

impl<T1, T2> ::yew::html::Component for Generic2<T1, T2>
where
    T1: 'static,
    T2: 'static,
{
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        unimplemented!()
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        unimplemented!()
    }}

fn compile_fail() {
    html! { <Generic<String>> };
    html! { <Generic<String>></Generic> };
    html! { <Generic<String>></Generic<Vec<String>>> };

    html! { <Generic<String>></Generic<std::path::Path>> };
    html! { <Generic<String>></> };
}

fn main() {}
