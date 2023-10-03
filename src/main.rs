#![deny(clippy::all)]
#![warn(
    clippy::use_self,
    clippy::cognitive_complexity,
    clippy::cloned_instead_of_copied,
    clippy::derive_partial_eq_without_eq,
    clippy::equatable_if_let,
    clippy::explicit_into_iter_loop,
    clippy::format_push_string,
    clippy::get_unwrap,
    clippy::match_same_arms,
    clippy::needless_for_each,
    clippy::todo
)]

use clap::{command, Parser};
use configuration::setup_configuration;
use databases::{setup_quasar_database, setup_stellar_node_database_connection};
use ingestion::ingest;
use logger::setup_logger;
use server::serve;

mod configuration;
mod databases;
mod ingestion;
mod logger;
mod schema;
mod server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Database URL to use as a backend
    #[arg(short, long)]
    database_url: Option<String>,

    /// Stellar node URL to ingest data from
    #[arg(short, long)]
    stellar_node_database_url: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let configuration = setup_configuration(args);

    setup_logger();

    let quasar_database = setup_quasar_database(&configuration).await;
    let node_database = setup_stellar_node_database_connection(&configuration).await;

    serve(&configuration.api, quasar_database.clone()).await;
    ingest(node_database, quasar_database, &configuration.ingestion).await;
}
