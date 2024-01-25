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
use configuration::{setup_configuration, Configuration};
use database_metrics::start_database_metrics;
use databases::{setup_quasar_database, QuasarDatabase};
use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    SinkExt, StreamExt,
};
use log::debug;
use logger::setup_logger;
use migration::OnConflict;
use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use prometheus::Registry;
use quasar_entities::{account, contract};
use sea_orm::EntityTrait;
use server::serve;
use stellar_xdr::curr::{
    self, BucketEntry, LedgerEntry, LedgerHeader, Limited, Limits, ReadXdr, Type, TypeVariant,
};
use tokio::spawn;

mod configuration;
mod database_metrics;
mod databases;
// mod ingestion;
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

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);
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
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                for path in event.paths {
                    match read_bucket_entry(&path) {
                        Err(e) => {
                            debug!("Invalid file {path:?}, Error: {e:?}");
                        }
                        Ok(res) => {
                            for ty in res {
                                sender.send(ty).await.unwrap();
                            }
                        }
                    }
                }
            }
            Err(e) => debug!("watch error: {:?}", e),
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
    let args = Args::parse();

    let configuration = setup_configuration(args);

    setup_logger();

    let quasar_database = setup_quasar_database(&configuration).await;

    let metrics = Registry::new();

    // Start a background task to collect database metrics
    // start_database_metrics(
    //     quasar_database.clone(),
    //     metrics.clone(),
    //     configuration.metrics.database_polling_interval,
    // );
    tokio::join!(
        setup_watcher(&quasar_database, &configuration),
        serve(&configuration.api, quasar_database.clone(), metrics.clone())
    );
}

async fn setup_watcher(db: &QuasarDatabase, cfg: &Configuration) {
    let data_dir = cfg.ingestion.buckets_path.clone();
    let (tx, mut rx) = channel(10);
    tokio::spawn(async move {
        async_watch(Path::new(&data_dir), tx).await.unwrap();
    });

    while let Some(n) = rx.next().await {
        println!("{n:?}");
        match n {
            Type::BucketEntry(entry) => match *entry {
                BucketEntry::Liveentry(e) | BucketEntry::Initentry(e) => match e.data {
                    curr::LedgerEntryData::Account(acc) => {
                        let account: account::ActiveModel =
                            account::ActiveModel::try_from(acc).unwrap();
                        account::Entity::insert(account)
                            .on_conflict(
                                OnConflict::column(account::Column::Id)
                                    .update_columns([
                                        account::Column::LastModified,
                                        account::Column::Balance,
                                        account::Column::BuyingLiabilities,
                                        account::Column::HomeDomain,
                                        account::Column::InflationDestination,
                                        account::Column::MasterWeight,
                                        account::Column::NumberOfSubentries,
                                        account::Column::SellingLiabilities,
                                        account::Column::SequenceNumber,
                                    ])
                                    .to_owned(),
                            )
                            .exec(db.as_inner())
                            .await
                            .unwrap();
                    }
                    curr::LedgerEntryData::ContractData(entry) => {
                        let contract: contract::ActiveModel =
                            contract::ActiveModel::try_from(entry).unwrap();
                        contract::Entity::insert(contract)
                            .on_conflict(
                                OnConflict::column(contract::Column::Address)
                                    .update_columns([
                                        contract::Column::LastModified,
                                        contract::Column::Hash,
                                        contract::Column::Key,
                                        contract::Column::Type,
                                    ])
                                    .to_owned(),
                            )
                            .exec(db.as_inner())
                            .await
                            .unwrap();
                    }
                    
                    _ => {
                        println!("Data {:?}", e.data);
                    } // curr::LedgerEntryData::Trustline(_) => todo!(),
                      // curr::LedgerEntryData::Offer(_) => todo!(),
                      // curr::LedgerEntryData::Data(_) => todo!(),
                      // curr::LedgerEntryData::ClaimableBalance(_) => todo!(),
                      // curr::LedgerEntryData::LiquidityPool(_) => todo!(),

                      // curr::LedgerEntryData::ConfigSetting(_) => todo!(),
                      // curr::LedgerEntryData::Ttl(_) => todo!(),
                },
                _ => {
                    println!("Entry {:?}", entry);
                }
            },
            _ => unreachable!("We should not get here."),
        }
    }
}
