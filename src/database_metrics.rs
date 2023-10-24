use std::time::Duration;

use prometheus::{IntGauge, Registry};
use quasar_entities::ledger;
use sea_orm::{EntityTrait, PaginatorTrait};
use tokio::time;

use crate::databases::QuasarDatabase;

pub(super) fn start_database_metrics(database: QuasarDatabase, registry: Registry, interval: u64) {
    tokio::spawn(async move {
        database_metrics(database, registry, interval).await;
    });
}

async fn database_metrics(database: QuasarDatabase, registry: Registry, interval: u64) {
    let mut ledger_gauge = setup_metrics(registry);
    let mut interval = time::interval(Duration::from_secs(interval));

    loop {
        interval.tick().await;

        count_entities(&database, &mut ledger_gauge).await;
    }
}

fn setup_metrics(
    registry: Registry,
) -> prometheus::core::GenericGauge<prometheus::core::AtomicI64> {
    let ledger_gauge = IntGauge::new("all_ledgers", "Number of ledgers in the database").unwrap();
    registry
        .register(Box::new(ledger_gauge.clone()))
        .expect("Failed to register counter");
    ledger_gauge
}

async fn count_entities(database: &QuasarDatabase, ledger_gauge: &mut IntGauge) {
    let ledger_count = ledger::Entity::find()
        .count(database.as_inner())
        .await
        .expect("Failed to count ledgers");

    ledger_gauge.set(ledger_count as i64);
}
