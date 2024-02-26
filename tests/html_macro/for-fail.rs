use ::yew_html_ext::html;

mod smth {
    const KEY: u32 = 42;
}

fn main() {
    _ = html!{for x in};
    _ = html!{for x in 0 .. 10};
    _ = html!{for (x, y) in 0 .. 10 {
        <span>{x}</span>
    }};

    _ = html!{for _ in 0 .. 10 {
        <div key="duplicate" />
    }};

    _ = html!{for _ in 0 .. 10 {
        <div key={smth::KEY} />
    }};
}
