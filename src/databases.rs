use std::ops::Deref;

use sea_orm::Database;

use log::{error, info};

use migration::{Migrator, MigratorTrait};

use sea_orm::DatabaseConnection;

use crate::configuration::Configuration;

#[derive(Clone)]
pub(super) struct QuasarDatabase(DatabaseConnection);

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

pub(super) async fn setup_quasar_database(configuration: &Configuration) -> QuasarDatabase {
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
        error!("Database URL not set. Use config/ or --database-url");
        std::process::exit(1);
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
            error!("Error connecting to {}, {}", database_url, error);
            std::process::exit(1);
        }
    }
}
