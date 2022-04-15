use std::{f32::consts::E, future};

use anyhow::Context;
use common::{CreatePoll, Poll, PollV1, PublicPollId, Rpc};
use jsonrpc_core::BoxFuture;
use jsonrpc_http_server::ServerBuilder;
use serde::Serialize;
use structopt::StructOpt;
struct Server {
    database: sled::Db,
}

fn create_poll(db: &sled::Db, poll: CreatePoll) -> anyhow::Result<Poll> {
    let id = PublicPollId::from_str(nanoid::nanoid!());
    let poll = Poll::V1(PollV1 {
        id: id.clone(),
        title: poll.title,
        description_text_markdown: poll.description_text_markdown,
        options: poll.options,
        votes: vec![],
    });
    let polls = db.open_tree("polls")?;
    polls.insert(&serde_cbor::to_vec(&id)?, serde_cbor::to_vec(&poll)?)?;
    Ok(poll)
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
