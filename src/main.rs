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

use quasar_indexer::configuration::{setup_configuration, Args};
use quasar_indexer::database_metrics::start_database_metrics;
use quasar_indexer::databases::{setup_quasar_database, setup_stellar_node_database};
use quasar_indexer::ingestion::ingest;
use quasar_indexer::logger::setup_logger;
use quasar_indexer::server::serve;

use clap::Parser;
use prometheus::Registry;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let configuration = setup_configuration(Some(args));

    setup_logger();

    let quasar_database = setup_quasar_database(&configuration).await;
    let node_database = setup_stellar_node_database(&configuration).await;

    let metrics = Registry::new();

    // Start a background task to collect database metrics
    start_database_metrics(
        quasar_database.clone(),
        metrics.clone(),
        configuration.metrics.database_polling_interval,
    );

    // Start the HTTP server, including GraphQL API
    serve(
        &configuration.api,
        quasar_database.clone(),
        Some(metrics.clone()),
    )
    .await;

    // Start the ingestion loop
    ingest(
        node_database,
        quasar_database,
        configuration.ingestion,
        Some(metrics),
    )
    .await;
}
