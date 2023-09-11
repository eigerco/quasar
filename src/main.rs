
use clap::{command, Parser};
use config::{Config, Environment, File};
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
    #[arg(long)]
    database_url: Option<String>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
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
    if let Some(database_url) = configuration.database_url {
        info!("Database URL: {}", database_url);
    } else {
        error!("No database URL configured, exiting...");
        std::process::exit(1);
    }
}
