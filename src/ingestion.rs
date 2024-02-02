use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    SinkExt, StreamExt,
};
use log::{debug, error, warn};
use prometheus::{IntCounter, Registry};
use quasar_entities::{account::AccountError, event::EventError};
use sea_orm::DbErr;
use std::{fs::File, path::Path};
use thiserror::Error;

use crate::{
    configuration::Configuration,
    databases::{setup_stellar_node_database, NodeDatabase, QuasarDatabase},
    ingestion::accounts::ingest_account,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use stellar_xdr::curr::{self, BucketEntry, Limited, Limits, Type, TypeVariant};

use self::{contracts::ingest_contract, transactions::ingest_transactions};

mod accounts;
mod contracts;
mod events;
// mod ledgers;
mod operations;
mod transactions;

#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("Database error: {0}")]
    Db(#[from] DbErr),
    #[error("XDR decoding error: {0}")]
    Xdr(#[from] stellar_xdr::curr::Error),
    #[error("Account error: {0}")]
    Account(#[from] AccountError),
    #[error("Event error: {0}")]
    Event(#[from] EventError),
}

#[derive(Debug, Clone)]
pub(super) struct IngestionMetrics {
    pub ledgers: IntCounter,
    pub accounts: IntCounter,
    pub contracts: IntCounter,
    pub transactions: IntCounter,
    pub operations: IntCounter,
    pub events: IntCounter,
}

pub async fn run_watcher(db: &QuasarDatabase, cfg: &Configuration, metrics: Registry) {
    let node_database = setup_stellar_node_database(cfg).await;
    let data_dir = cfg.ingestion.buckets_path.clone();
    let (tx, mut rx) = channel(10);
    tokio::spawn(async move {
        async_watch(Path::new(&data_dir), tx).await.unwrap();
    });

    let ingestion_metrics: IngestionMetrics = setup_ingestion_metrics(&metrics);

    while let Some(bucket) = rx.next().await {
        if let Err(e) = ingest_next(db, &node_database, bucket, ingestion_metrics.clone()).await {
            ingestion_metrics.ledgers.inc();
            warn!("Ingestion error: {e:?}")
        }
    }
}

async fn ingest_next(
    db: &QuasarDatabase,
    node_db: &NodeDatabase,
    bucket: Type,
    metrics: IngestionMetrics,
) -> Result<(), IngestionError> {
    match bucket {
        Type::BucketEntry(entry) => match *entry {
            BucketEntry::Liveentry(e) | BucketEntry::Initentry(e) => {
                let ledger_sequence = e.last_modified_ledger_seq;

                match e.data {
                    curr::LedgerEntryData::Account(acc) => {
                        ingest_account(db, acc).await?;
                        metrics.accounts.inc();
                    }

                    curr::LedgerEntryData::ContractData(contract) => {
                        ingest_contract(db, contract).await?;
                        metrics.contracts.inc();
                    }

                    _ => {
                        log::trace!("Unprocessed ledger entry: {:?}", e);
                    }
                }
                ingest_transactions(node_db, db, ledger_sequence, &metrics)
                    .await
                    .unwrap();
            }
            _ => {
                log::trace!("Metadata {:?}", entry);
            }
        },
        _ => unreachable!("We should not get here."),
    }

    Ok(())
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

fn setup_ingestion_metrics(metrics: &Registry) -> IngestionMetrics {
    let ledgers = create_ingestion_counter(metrics, "ledgers");
    let contracts = create_ingestion_counter(metrics, "contracts");
    let accounts = create_ingestion_counter(metrics, "accounts");
    let transactions = create_ingestion_counter(metrics, "transactions");
    let operations = create_ingestion_counter(metrics, "operations");
    let events = create_ingestion_counter(metrics, "events");

    IngestionMetrics {
        ledgers,
        contracts,
        accounts,
        transactions,
        operations,
        events,
    }
}

fn create_ingestion_counter(
    metrics: &Registry,
    name: &str,
) -> prometheus::core::GenericCounter<prometheus::core::AtomicU64> {
    let counter = IntCounter::new(
        format!("ingested_{}", name),
        format!("Number of ingested {}", name),
    )
    .unwrap();
    metrics
        .register(Box::new(counter.clone()))
        .expect("Failed to register counter");
    counter
}
