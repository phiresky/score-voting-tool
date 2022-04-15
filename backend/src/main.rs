use std::{collections::HashMap, f32::consts::E, future};

use anyhow::Context;
use common::{CreatePoll, Poll, PollOptionId, PollV1, PublicPollId, Rpc, ScoreVote};
use jsonrpc_core::BoxFuture;
use jsonrpc_http_server::ServerBuilder;
use serde::Serialize;
use sled::transaction::{
    ConflictableTransactionResult, TransactionError, TransactionResult, TransactionalTree,
};
use structopt::StructOpt;
struct Server {
    database: sled::Db,
}

#[derive(Debug, Serialize)]
pub struct OurError {
    // todo: better variants
    msg: String,
}

impl From<OurError> for jsonrpc_core::Error {
    fn from(e: OurError) -> Self {
        jsonrpc_core::Error {
            code: jsonrpc_core::ErrorCode::ServerError(1),
            message: e.msg,
            data: None,
        }
    }
}
impl From<anyhow::Error> for OurError {
    fn from(e: anyhow::Error) -> Self {
        OurError { msg: e.to_string() }
    }
}

// Server implementation
impl Rpc<OurError> for Server {
    fn add(&self, a: u64, b: u64) -> Result<u64, OurError> {
        Ok(a + b)
    }

    fn create_poll(&self, poll: CreatePoll) -> Result<Poll, OurError> {
        let id = PublicPollId::from_str(nanoid::nanoid!());
        let poll = Poll::V1(PollV1 {
            id: id.clone(),
            title: poll.title,
            description_text_markdown: poll.description_text_markdown,
            options: poll.options,
            votes: vec![],
            result: None,
        });
        let polls = self
            .database
            .open_tree("polls")
            .context("opening database")?;
        polls
            .insert(
                &serde_cbor::to_vec(&id).context("serializing")?,
                serde_cbor::to_vec(&poll).context("serializing")?,
            )
            .context("inserting into db")?;
        Ok(poll)
    }

    fn get_poll(&self, id: PublicPollId) -> Result<Poll, OurError> {
        let polls = self
            .database
            .open_tree("polls")
            .context("opening database")?;
        let id = &serde_cbor::to_vec(&id).context("serializing")?;
        let poll_ser = polls
            .get(&id)
            .context("loading")?
            .context("poll not found")?;
        Ok(serde_cbor::from_slice::<Poll>(&poll_ser).context("deserializing")?)
    }

    fn call(&self, _: u64) -> BoxFuture<Result<String, OurError>> {
        Box::pin(future::ready(Ok("OK".to_owned())))
    }

    fn vote(&self, poll_id: PublicPollId, vote: common::ScoreVote) -> Result<Poll, OurError> {
        let polls = self
            .database
            .open_tree("polls")
            .context("opening database")?;
        let poll = polls
            .transaction(
                |polls: &TransactionalTree| -> ConflictableTransactionResult<Poll, anyhow::Error> {
                    use sled::transaction::ConflictableTransactionError::Abort;
                    let id_ser = serde_cbor::to_vec(&poll_id)
                        .context("serializing")
                        .map_err(Abort)?;
                    let mut poll = {
                        let poll_ser = polls
                            .get(&id_ser)
                            .context("loading")
                            .map_err(Abort)?
                            .context("poll not found")
                            .map_err(Abort)?;
                        let x = serde_cbor::from_slice::<Poll>(&poll_ser)
                            .context("deserializing")
                            .map_err(Abort)?;
                        let Poll::V1(p) = x;
                        p
                    };
                    poll.votes.push(vote.clone());
                    poll.result = Some(compute_vote_result(&poll.votes));
                    let poll = Poll::V1(poll);
                    let ser = serde_cbor::to_vec(&poll)
                        .context("serializing")
                        .map_err(Abort)?;
                    polls.insert(id_ser, ser)?;
                    Ok(poll)
                },
            )
            .map_err(|e| match e {
                TransactionError::Abort(e) => e,
                TransactionError::Storage(e) => anyhow::anyhow!("sled error: {e}"),
            })
            .context("error in transaction")?;
        Ok(poll)
    }
}

fn compute_vote_result(
    votes: &[ScoreVote],
) -> std::collections::HashMap<PollOptionId, Option<f64>> {
    let mut map: HashMap<PollOptionId, Vec<f64>> = HashMap::new();
    for vote in votes {
        for (id, r) in &vote.votes {
            if let Some(r) = r {
                map.entry(id.clone()).or_insert_with(|| vec![]).push(*r);
            }
        }
    }
    map.into_iter()
        .map(|(k, v)| (k, Some(v.iter().sum::<f64>() / (v.len() as f64))))
        .collect()
}

#[derive(StructOpt)]
#[structopt()]
enum Commands {
    Start { listen: String },
    Dump {},
}

fn main() -> anyhow::Result<()> {
    match Commands::from_args() {
        Commands::Start { listen } => {
            let mut io = jsonrpc_core::IoHandler::new();
            let rpc_server = Server {
                database: sled::open("server-database.sled")?,
            };
            io.extend_with(rpc_server.to_delegate());
            let jsonrpc_server = ServerBuilder::new(io)
                .threads(3)
                .start_http(
                    &listen
                        .parse()
                        .context("could not parse listen address (format: 127.0.0.1:3030)")?,
                )
                .unwrap();

            jsonrpc_server.wait();
            Ok(())
        }
        Commands::Dump {} => {
            let db = sled::open("server-database.sled")?;
            let tree = db.open_tree("polls")?;
            for ele in tree.iter() {
                let (_k, v) = ele?;
                let poll: Poll = serde_cbor::from_slice(&v)?;
                println!("{}", serde_json::to_string_pretty(&poll)?);
            }
            Ok(())
        }
    }
}
