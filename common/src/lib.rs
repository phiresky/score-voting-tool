use jsonrpc_derive::rpc;
use jsonrpc_core::{BoxFuture, Result};


/// Rpc trait
#[rpc]
pub trait Rpc {
    /// Adds two numbers and returns a result
    #[rpc(name = "add")]
    fn add(&self, a: u64, b: u64) -> Result<u64>;

    /// Performs asynchronous operation
    #[rpc(name = "callAsync")]
    fn call(&self, a: u64) -> BoxFuture<Result<String>>;
}

pub use gen_client::Client as ApiClient;