//! defines the isomorphic code (common to both client and server)
use std::collections::HashMap;

use jsonrpc_core::BoxFuture;
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};
use std::result::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicUserId(String);
impl PublicUserId {
    pub fn from_str(str: impl Into<String>) -> PublicUserId {
        PublicUserId(str.into())
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicPollId(String);

impl PublicPollId {
    pub fn from_str(str: impl Into<String>) -> PublicPollId {
        PublicPollId(str.into())
    }
    pub fn to_str(&self) -> &str {
        &self.0
    }
}
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PollOptionId(String);
impl PollOptionId {
    pub fn from_str(str: String) -> PollOptionId {
        PollOptionId(str)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PollV1 {
    pub id: PublicPollId,
    pub title: String,
    pub description_text_markdown: String,
    pub options: Vec<PollOption>,
    pub votes: Vec<ScoreVote>,
    pub result: Option<HashMap<PollOptionId, Option<f64>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PollOption {
    pub id: PollOptionId,
    pub title: String,
    pub description_text_markdown: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScoreVote {
    pub user_id: PublicUserId,
    pub user_name: String,
    pub votes: HashMap<PollOptionId, Option<f64>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Poll {
    V1(PollV1),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePoll {
    pub title: String,
    pub description_text_markdown: String,
    pub options: Vec<PollOption>,
}

#[rpc]
pub trait Rpc<ErrT>
where
    ErrT: Into<jsonrpc_core::Error>,
{
    /// Adds two numbers and returns a result
    #[rpc(name = "add")]
    fn add(&self, a: u64, b: u64) -> Result<u64, ErrT>;

    #[rpc(name = "create_poll")]
    fn create_poll(&self, poll: CreatePoll) -> Result<Poll, ErrT>;

    #[rpc(name = "get_poll")]
    fn get_poll(&self, poll_id: PublicPollId) -> Result<Poll, ErrT>;

    #[rpc(name = "vote")]
    fn vote(&self, poll_id: PublicPollId, vote: ScoreVote) -> Result<Poll, ErrT>;

    /// Performs asynchronous operation
    #[rpc(name = "callAsync")]
    fn call(&self, a: u64) -> BoxFuture<Result<String, ErrT>>;
}

pub type ApiClient = gen_client::Client<jsonrpc_core::types::error::Error>; // todo: a specific error type can't be preserved apparently
