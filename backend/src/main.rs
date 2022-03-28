use std::future;

use common::{CreatePoll, Poll, PollV1, PublicPollId, Rpc};
use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_http_server::ServerBuilder;

struct RpcImpl;

// Server implementation
impl Rpc for RpcImpl {
    fn add(&self, a: u64, b: u64) -> Result<u64> {
        Ok(a + b)
    }

    fn create_poll(&self, poll: CreatePoll) -> Result<Poll> {
        let out = Poll::V1(PollV1 {
            id: PublicPollId::from_str(nanoid::nanoid!()),
            title: poll.title,
            description_text_markdown: poll.description_text_markdown,
            options: poll.options,
            votes: vec![],
        });
        Ok(out)
    }

	fn get_poll(&self, id: PublicPollId) -> Result<Poll> {
		panic!()
	}

    fn call(&self, _: u64) -> BoxFuture<Result<String>> {
        Box::pin(future::ready(Ok("OK".to_owned())))
    }
}

fn main() {
    let mut io = jsonrpc_core::IoHandler::new();
    io.extend_with(RpcImpl.to_delegate());
    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .unwrap();

    server.wait();
}
