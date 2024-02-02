use std::ops::Deref;

use sea_orm::Database;

use log::info;

use migration::{Migrator, MigratorTrait};

use sea_orm::DatabaseConnection;

use crate::configuration::Configuration;

#[derive(Clone)]
pub struct NodeDatabase(DatabaseConnection);
#[derive(Clone)]
pub struct QuasarDatabase(DatabaseConnection);

impl Deref for NodeDatabase {
    type Target = DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NodeDatabase {
    pub fn as_inner(&self) -> &DatabaseConnection {
        &self.0
    }
}

impl Deref for QuasarDatabase {
    type Target = DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl QuasarDatabase {
    pub fn as_inner(&self) -> &DatabaseConnection {
        &self.0
    }
}

pub async fn setup_quasar_database(configuration: &Configuration) -> QuasarDatabase {
    let quasar_database = setup_quasar_database_connection(configuration).await;
    Migrator::up(&quasar_database, None)
        .await
        .expect("Migration failed");

    QuasarDatabase(quasar_database)
}

async fn setup_quasar_database_connection(configuration: &Configuration) -> DatabaseConnection {
    if let Some(database_url) = &configuration.quasar_database_url {
        info!("Connecting the backend database: {}", database_url);

        setup_connection(database_url).await
    } else {
        panic!("Database URL not set. Use config/ or --database-url");
    }
}

pub async fn setup_stellar_node_database(configuration: &Configuration) -> NodeDatabase {
    if let Some(node_database_url) = &configuration.stellar_node_database_url {
        info!(
            "Connecting the Stellar node database: {}",
            node_database_url
        );

        NodeDatabase(setup_connection(node_database_url).await)
    } else {
        panic!("Node database URL not set. Use config/, -s or --stellar-node-database-url");
    }
}

async fn setup_connection(database_url: &String) -> DatabaseConnection {
    let connection_result = Database::connect(database_url).await;

    match connection_result {
        Ok(connection) => {
            info!("Database connected");

            connection
        }
        Err(error) => {
            panic!("Error connecting to {}, {}", database_url, error);
        }
    }
}
