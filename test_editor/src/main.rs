use log;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use yew::prelude::*;

#[derive(Serialize, Deserialize)]
struct Message {
    value: String,
    typ: String,
    identifier: u64,
}

#[function_component(App)]
fn app() -> Html {
    let dialog_value = use_state(|| "".to_string());
    let counter = use_state(|| 0 as u64);
    let last_counter = use_state(|| 0 as u64);

    let mut client = wasm_sockets::EventClient::new("ws://0.0.0.0:8765")
        .expect("It was not possible to initialize the websocket");
    client.set_on_error(Some(Box::new(|error| {
        log::error!("{:#?}", error);
    })));
    client.set_on_connection(Some(Box::new(|client: &wasm_sockets::EventClient| {
        log::info!("{:#?}", client.status);
    })));
    client.set_on_close(Some(Box::new(|_evt| {
        log::info!("Connection closed");
    })));
    client.set_on_message(Some((|| {
        let dialog_value = dialog_value.clone();
        Box::new(
            move |client: &wasm_sockets::EventClient, message: wasm_sockets::Message| {
                log::info!("New Message: {:#?}", message);
                match message {
                    wasm_sockets::Message::Text(value) => {
                        let received_mess: Message = serde_json::from_str(value.as_str())
                            .expect("Message received has a incorrect format");
                        dialog_value.set(received_mess.value);
                    }
                    _ => (),
                }
            },
        )
    })()));

    let onchange = Callback::from({
        let dialog_value = dialog_value.clone();
        let counter = counter.clone();
        move |input_event: InputEvent| {
            if let Some(content) = input_event.data() {
                log::info!("{:}", content);
                let all_message = (*dialog_value).clone() + &content.clone();
                let messg = Message {
                    value: all_message.clone(),
                    typ: "text".to_string(),
                    identifier: *counter,
                };
                client.send_string(
                    &serde_json::to_string(&messg).expect("Message was not well built"),
                );

                counter.set(*counter + 1);
            }
        }
    });
    html! {
        <div>
                <textarea id="editor" oninput={onchange}></textarea>
                <textarea id="dialog" disabled=true value={(*dialog_value).clone()}> </textarea>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
