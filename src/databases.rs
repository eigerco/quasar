use sea_orm::Database;

use log::{error, info};

use migration::{Migrator, MigratorTrait};

use sea_orm::DatabaseConnection;

use crate::configuration::Configuration;

pub(super) async fn setup_quasar_database(configuration: &Configuration) -> DatabaseConnection {
    let quasar_database = setup_quasar_database_connection(configuration).await;
    Migrator::up(&quasar_database, None)
        .await
        .expect("Migration failed");

    quasar_database
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

pub(super) async fn setup_stellar_node_database_connection(
    configuration: &Configuration,
) -> DatabaseConnection {
    if let Some(node_database_url) = &configuration.stellar_node_database_url {
        info!(
            "Connecting the Stellar node database: {}",
            node_database_url
        );

        setup_connection(node_database_url).await
    } else {
        error!("Node database URL not set. Use config/, -s or --stellar-node-database-url");
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
