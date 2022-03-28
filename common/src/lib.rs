//! defines the isomorphic code (common to both client and server)
use std::collections::HashMap;

use jsonrpc_core::{BoxFuture, Result};
use serde::{Deserialize, Serialize};
use jsonrpc_derive::rpc;

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicUserId(String);
#[derive(Serialize, Deserialize, Debug)]
pub struct PublicPollId(String);

impl PublicPollId {
	pub fn from_str(str: String) -> PublicPollId {
		PublicPollId(str)
	}
}
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
pub struct PollOptionId(String);

#[derive(Serialize, Deserialize, Debug)]
pub struct PollV1 {
   pub id: PublicPollId,
   pub title: String,
   pub description_text_markdown: String,
   pub options: Vec<PollOption>,
   pub votes: Vec<ScoreVote>,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct PollOption {
    pub id: PollOptionId,
    pub title: String,
    pub description_text_markdown: String,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct ScoreVote {
   pub  user_id: PublicUserId,
    pub votes: HashMap<PollOptionId, f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Poll {
    V1(PollV1),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePoll {
    pub title: String,
    pub description_text_markdown: String,
    pub options: Vec<PollOption>
}

#[rpc]
pub trait Rpc {
    /// Adds two numbers and returns a result
    #[rpc(name = "add")]
    fn add(&self, a: u64, b: u64) -> Result<u64>;

    #[rpc(name = "create_poll")]
    fn create_poll(&self, poll: CreatePoll) -> Result<Poll>;

    #[rpc(name="get_poll")]
    fn get_poll(&self, poll_id: PublicPollId) -> Result<Poll>;
    
    /// Performs asynchronous operation
    #[rpc(name = "callAsync")]
    fn call(&self, a: u64) -> BoxFuture<Result<String>>;
}

pub use gen_client::Client as ApiClient;
