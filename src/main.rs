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

use clap::Parser;
use logger::setup_logger;

use prometheus::Registry;
use server::serve;

use crate::{configuration::setup_configuration, database_metrics::start_database_metrics, databases::setup_quasar_database};

mod configuration;
mod database_metrics;
mod databases;
mod ingestion;
mod logger;
mod metrics;
mod schema;
mod server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Database URL to use as a backend
    #[arg(short, long)]
    database_url: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let configuration = setup_configuration(args);

    setup_logger();

    let quasar_database = setup_quasar_database(&configuration).await;

    let metrics = Registry::new();

    // Start a background task to collect database metrics
    start_database_metrics(
        quasar_database.clone(),
        metrics.clone(),
        configuration.metrics.database_polling_interval,
    );
    tokio::join!(
        ingestion::run_watcher(&quasar_database, &configuration, metrics.clone()),
        serve(&configuration.api, quasar_database.clone(), metrics.clone())
    );
}
