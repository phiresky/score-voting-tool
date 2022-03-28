use jsonrpc_core_client::transports::wasmhttp;
use yew::prelude::*;

pub async fn connect() -> common::ApiClient {
    let (client, receiver_task) = wasmhttp::connect::<common::ApiClient>("http://localhost:3030/")
        .await
        .expect("shouldn't fail");
    wasm_bindgen_futures::spawn_local(receiver_task);
    client
}

#[function_component(App)]
fn test_app() -> Html {
    let sum = use_state(|| 0);
    {
        let sum = sum.clone();
        use_effect_with_deps(
            move |_| {
                let videos = sum.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let client = connect().await; // todo: connect only once
                    videos.set(client.add(1, 2).await.expect("todo: handle"));
                });
                || ()
            },
            (),
        );
    }
    html! {
        <div>{"hello. result of 1 + 2 = "}{*sum}</div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Hello, world!");
    yew::start_app::<App>();
}
