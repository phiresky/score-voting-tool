use jsonrpc_core_client::transports::wasmhttp;

pub async fn connect() -> common::ApiClient {
    let (client, receiver_task) = wasmhttp::connect::<common::ApiClient>("http://localhost:3030/")
        .await
        .expect("fooooo");
    wasm_bindgen_futures::spawn_local(receiver_task);
    client
}
