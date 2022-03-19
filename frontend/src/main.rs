use yew::prelude::*;
mod api_client;

#[function_component(App)]
fn test_app() -> Html {
    let videos = use_state(|| 0);
    {
        let videos = videos.clone();
        use_effect_with_deps(
            move |_| {
                let videos = videos.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let client = crate::api_client::connect().await;
                    videos.set(client.add(1, 2).await.expect("NOP"));
                });
                || ()
            },
            (),
        );
    }
    html! {
        <div>{"hello. result of 1 + 2 = "}{*videos}</div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Hello, world!");
    yew::start_app::<App>();
}
