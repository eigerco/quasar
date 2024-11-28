use std::{collections::HashMap, time::Duration};

use log::error;
use prometheus::{IntGauge, Registry};
use quasar_entities::prelude::*;
use sea_orm::{EntityTrait, PaginatorTrait};
use tokio::time;

use crate::databases::QuasarDatabase;

const GAUGES: [&str; 6] = [
    "ledgers",
    "accounts",
    "contracts",
    "transactions",
    "operations",
    "events",
];

pub fn start_database_metrics(database: QuasarDatabase, registry: Registry, interval: u64) {
    tokio::spawn(async move {
        database_metrics(database, registry, interval).await;
    });
}

async fn database_metrics(database: QuasarDatabase, registry: Registry, interval: u64) {
    let mut gauges = setup_metrics(registry);
    let mut interval = time::interval(Duration::from_secs(interval));

    loop {
        count_entities(&database, &mut gauges).await;

        interval.tick().await;
    }
}

fn setup_metrics(registry: Registry) -> HashMap<String, IntGauge> {
    let mut gauges = HashMap::new();

    for gauge_name in GAUGES {
        let gauge = IntGauge::new(
            gauge_name,
            format!("Number of {} in the database", gauge_name),
        )
        .unwrap();
        registry
            .register(Box::new(gauge.clone()))
            .expect("Failed to register counter");

        gauges.insert(gauge_name.to_string(), gauge);
    }

    gauges
}

async fn count_entities(db: &QuasarDatabase, gauges: &mut HashMap<String, IntGauge>) {
    let db = db.as_inner();

    for (gauge_name, gauge) in gauges {
        let query = match gauge_name.as_str() {
            "ledgers" => Ledger::find().count(db),
            "accounts" => Account::find().count(db),
            "contracts" => Contract::find().count(db),
            "transactions" => Transaction::find().count(db),
            "operations" => Operation::find().count(db),
            "events" => Event::find().count(db),
            _ => panic!("Unknown gauge name"),
        };

        let result = query.await;

        if let Ok(count) = result {
            gauge.set(count as i64);
            continue;
        } else {
            error!("Failed to count entities: {:?}", result);
        }
    }
}
