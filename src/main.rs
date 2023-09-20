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

use clap::{command, Parser};
use config::{Config, Environment, File};
use diesel::{Connection, PgConnection};
use log::{error, info, LevelFilter};
use log4rs::{
    append::console::ConsoleAppender,
    config::Config as Log4rsConfig,
    config::{Appender, Root},
};

use crate::configuration::Configuration;

mod configuration;

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

fn setup_logger() {
    let stdout = ConsoleAppender::builder().build();

    let config = Log4rsConfig::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();
}

fn setup_configuration(args: Args) -> Configuration {
    let mut config_builder = Config::builder()
        .add_source(File::with_name("config/config"))
        .add_source(File::with_name("config/local").required(false))
        .add_source(Environment::with_prefix("app"));

    if let Some(database_url) = args.database_url {
        config_builder = config_builder
            .set_override("database_url", database_url)
            .expect("Failed to set database_url");
    }

    if let Some(stellar_node_database_url) = args.stellar_node_database_url {
        config_builder = config_builder
            .set_override("stellar_node_database_url", stellar_node_database_url)
            .expect("Failed to set stellar_node_database_url");
    }

    let configuration = config_builder
        .build()
        .expect("Failed to build configuration");
    let configuration: Configuration = configuration
        .try_deserialize()
        .expect("Failed to deserialize configuration");
    configuration
}

fn main() {
    let args = Args::parse();

    let configuration = setup_configuration(args);

    setup_logger();

    start(configuration);
}

fn start(configuration: Configuration) {
    let _database_connection = setup_database_connection(&configuration);
    let _node_database_connection = setup_stellar_node_database_connection(&configuration);
}

fn setup_database_connection(configuration: &Configuration) -> PgConnection {
    if let Some(database_url) = &configuration.database_url {
        info!("Connecting the backend database: {}", database_url);

        setup_connection(database_url)
    } else {
        error!("Database URL not set. Use config/ or --database-url");
        std::process::exit(1);
    }
}

fn setup_stellar_node_database_connection(configuration: &Configuration) -> PgConnection {
    if let Some(node_database_url) = &configuration.stellar_node_database_url {
        info!(
            "Connecting the Stellar node database: {}",
            node_database_url
        );

        setup_connection(node_database_url)
    } else {
        error!("Node database URL not set. Use config/, -s or --stellar-node-database-url");
        std::process::exit(1);
    }
}

fn setup_connection(database_url: &String) -> PgConnection {
    let connection_result = PgConnection::establish(database_url);

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
