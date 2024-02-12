use yew_html_ext::html;

fn compile_fail() {
    html! {
        <>
            { () }
        </>
    };

    let not_tree = || ();
    html! {
        <div>{ not_tree() }</div>
    };
    html! {
        <>{ for (0..3).map(|_| not_tree()) }</>
    };
}

fn main() {}
