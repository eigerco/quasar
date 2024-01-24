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

use std::{fs::File, io, path::Path};

use clap::{command, Parser};
use configuration::setup_configuration;
use database_metrics::start_database_metrics;
use databases::{setup_quasar_database, setup_stellar_node_database};
use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    SinkExt, StreamExt,
};
use ingestion::ingest;
use logger::setup_logger;
use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use prometheus::Registry;
use server::serve;
use stellar_xdr::curr::{
    self, BucketEntry, LedgerEntry, LedgerHeader, Limited, Limits, ReadXdr, Type, TypeVariant,
};
use tokio::spawn;

mod bytes;
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

    /// Stellar node URL to ingest data from
    #[arg(short, long)]
    stellar_node_database_url: Option<String>,
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            });
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn async_watch<P: AsRef<Path>>(path: P, mut sender: Sender<Type>) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                let path = &event.paths[0];
                match read_bucket_entry(&path) {
                    Err(e) => {
                        // println!("Invalid file {path:?}, Error: {e:?}");
                    }
                    Ok(res) => {
                        for ty in res {
                            sender.send(ty).await.unwrap();
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

fn read_bucket_entry(file: &Path) -> Result<Vec<Type>, curr::Error> {
    match File::open(file) {
        Ok(res) => {
            let mut f = Limited::new(Box::new(res), Limits::none());
            Type::read_xdr_framed_iter(TypeVariant::BucketEntry, &mut f).collect()
        }
        Err(err) => Err(curr::Error::Io(err)),
    }
}

enum Entity {
    Account,
    TrustLine,
    Offer,
    Data,
    ContractCode,
    ContractData,
    // Not resolved, but are related and should need TX to be decodable
    Transaction,
    Operation,
    Event,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = channel(10);
    tokio::spawn(async move {
        async_watch(Path::new("./datadir/core/buckets"), tx)
            .await
            .unwrap();
    });

    while let Some(n) = rx.next().await {
        println!("{n:?}");
        // match n {
        //     Type::BucketEntry(entry) => match entry {
        //         BucketEntry::Liveentry(e) => match e.data {
        //             curr::LedgerEntryData::Account(_) => todo!(),
        //             curr::LedgerEntryData::Trustline(_) => todo!(),
        //             curr::LedgerEntryData::Offer(_) => todo!(),
        //             curr::LedgerEntryData::Data(_) => todo!(),
        //             curr::LedgerEntryData::ClaimableBalance(_) => todo!(),
        //             curr::LedgerEntryData::LiquidityPool(_) => todo!(),
        //             curr::LedgerEntryData::ContractData(_) => todo!(),
        //             curr::LedgerEntryData::ContractCode(_) => todo!(),
        //             curr::LedgerEntryData::ConfigSetting(_) => todo!(),
        //             curr::LedgerEntryData::Ttl(_) => todo!(),
        //         },
        //         BucketEntry::Initentry(_) => todo!(),
        //         BucketEntry::Deadentry(_) => todo!(),
        //         BucketEntry::Metaentry(_) => {}
        //     },
        //     _ => unreachable!("We should not get here."),
        // }
    }
    // let args = Args::parse();

    // let configuration = setup_configuration(args);

    // setup_logger();

    // let quasar_database = setup_quasar_database(&configuration).await;
    // let node_database = setup_stellar_node_database(&configuration).await;

    // let metrics = Registry::new();

    // // Start a background task to collect database metrics
    // start_database_metrics(
    //     quasar_database.clone(),
    //     metrics.clone(),
    //     configuration.metrics.database_polling_interval,
    // );

    // // Start the HTTP server, including GraphQL API
    // serve(&configuration.api, quasar_database.clone(), metrics.clone()).await;

    // // Start the ingestion loop
    // ingest(
    //     node_database,
    //     quasar_database,
    //     configuration.ingestion,
    //     metrics,
    // )
    // .await;
}
