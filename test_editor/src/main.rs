use log;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let onchange = Callback::from(move |input_event: InputEvent| {
        if let Some(content) = input_event.data() {
            log::info!("{:}", content);
        }
    });
    html! {
        <div>
                <textarea id="editor" oninput={onchange}></textarea>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
