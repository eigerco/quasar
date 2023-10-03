use config::{Config, Environment, File};
use serde::Deserialize;

use crate::Args;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Ingestion {
    pub polling_interval: u64,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Api {
    pub host: String,
    pub port: u16,

    pub depth_limit: usize,
    pub complexity_limit: usize,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Configuration {
    pub quasar_database_url: Option<String>,
    pub stellar_node_database_url: Option<String>,

    pub ingestion: Ingestion,
    pub api: Api,
}

pub(super) fn setup_configuration(args: Args) -> Configuration {
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
