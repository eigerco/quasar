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
use configuration::setup_configuration;
use database_metrics::start_database_metrics;
use databases::{setup_quasar_database, setup_stellar_node_database};
use ingestion::ingest;
use logger::setup_logger;
use prometheus::Registry;
use server::serve;

mod configuration;
mod database_metrics;
mod databases;
mod ingestion;
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

    /// Stellar node URL to ingest data from
    #[arg(short, long)]
    stellar_node_database_url: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let configuration = setup_configuration(Some(args));

    setup_logger();

    let quasar_database = setup_quasar_database(&configuration).await;
    let node_database = setup_stellar_node_database(&configuration).await;

    let metrics = Registry::new();

    // Start a background task to collect database metrics
    start_database_metrics(
        quasar_database.clone(),
        metrics.clone(),
        configuration.metrics.database_polling_interval,
    );

    // Start the HTTP server, including GraphQL API
    serve(&configuration.api, quasar_database.clone(), metrics.clone()).await;

    // Start the ingestion loop
    ingest(
        node_database,
        quasar_database,
        configuration.ingestion,
        metrics,
    )
    .await;
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::env;
    use std::process::Command;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;

    use tokio::time::{sleep, Duration};
    use dockertest::waitfor::{MessageSource, MessageWait};
    use dockertest::{DockerTest, Image, TestBodySpecification};

    const MAGIC_LINE: &str = "postgres password: ";
    const PLAYGROUND_URL: &str = "http://127.0.0.1:8000";

    fn quasar_database_spec() -> TestBodySpecification {
        let exit_wait = MessageWait {
            message: "database system is ready to accept connections".to_string(),
            source: MessageSource::Stderr,
            timeout: 5,
        };

        let mut quasar_proc = TestBodySpecification::with_repository("postgres")
            .set_wait_for(Box::new(exit_wait))
            .set_handle("quasar");

        quasar_proc.modify_port_map(5432, 5432);
        quasar_proc.modify_named_volume("postgres-data", "/var/lib/postgresql/");
        quasar_proc.modify_env("POSTGRES_PASSWORD", "quasar");
        quasar_proc.modify_env("POSTGRES_DB", "quasar_development");

        quasar_proc
    }

    fn stellar_node_spec() -> TestBodySpecification {
        let exit_wait = MessageWait {
            message: MAGIC_LINE.to_string(),
            source: MessageSource::Stdout,
            timeout: 15,
        };

        let image = Image::with_repository("stellar/quickstart").tag("soroban-dev");

        let mut soroban_proc = TestBodySpecification::with_image(image)
            .set_wait_for(Box::new(exit_wait))
            .set_handle("stellar");

        soroban_proc.modify_port_map(5432, 8001);
        soroban_proc.append_cmd("--local");
        soroban_proc.append_cmd("--enable-soroban-rpc");

        soroban_proc
    }

    fn get_password_from_logs(container_name: &str) -> String {
        let out = Command::new("docker")
            .arg("logs")
            .arg(container_name)
            .output()
            .unwrap();

        println!("out: {:?}", out);

        let output = std::str::from_utf8(&out.stdout).unwrap();

        let remind = output.split(MAGIC_LINE).collect::<Vec<&str>>();
        let password = remind.get(1).unwrap().split("\n").collect::<Vec<&str>>();

        password.get(0).unwrap().to_string()
    }

    #[test]
    fn localhost_appears() {
        // Define our test instance, will pull images from dockerhub.
        let mut test = DockerTest::new();

        test.provide_container(quasar_database_spec())
            .provide_container(stellar_node_spec());

        let curr_dir = env::current_dir().unwrap();
        let mut path_str = curr_dir.display().to_string();
        path_str.push_str("/tmp");

        // Run a command.
        println!("containers provided..");

        let has_ran: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
        let has_ran_test = has_ran.clone();

        test.run(|ops| async move {
            // A handle to operate on the Container.
            let _container = ops.handle("quasar");
            let container2 = ops.handle("stellar");

            let mut configuration = setup_configuration(None);

            println!("config setup..");
            let quasar_database = setup_quasar_database(&configuration).await;
            println!("connected to quasar database..");

            let password = get_password_from_logs(container2.name());
            let conn_str = format!("postgres://stellar:{password}@localhost:8001/core");
            println!("stellar conn_str: {}", conn_str);
            configuration.stellar_node_database_url = Some(conn_str);

            // This mitigates slightly the problem below
            sleep(Duration::from_secs(4)).await;

            // This randomly throws the error:
            // Connection Error: encountered unexpected or invalid data: unexpected response from SSLRequest: 0x00
            // and sometimes that the password is wrong but that disappeared after some repeated runs
            let node_database = setup_stellar_node_database(&configuration).await;

            // run tests

            let metrics = Registry::new();

            // Start the HTTP server, including GraphQL API
            serve(&configuration.api, quasar_database.clone(), metrics.clone()).await;

            println!("started the http server");

            tokio::spawn(async move {
                // Start the ingestion loop
                ingest(
                    node_database,
                    quasar_database,
                    configuration.ingestion,
                    metrics,
                )
                .await;
            });

            println!("after ingest!");

            let res = reqwest::get(PLAYGROUND_URL).await.unwrap();

            assert_eq!(res.status(), reqwest::StatusCode::OK);

            let client = reqwest::Client::new();

            let mut query = HashMap::new();
            query.insert("operationName", None);
            // query.insert("variables", Some(&binding[..])); // how to write this??
            query.insert("query", Some("{\n  ledgers {\n    hash\n  }\n}\n"));

            let req = client.post(PLAYGROUND_URL).json(&query);

            let resp = req.send().await.unwrap();

            assert_eq!(res.status(), reqwest::StatusCode::OK);

            let text = resp.text().await.unwrap();

            let mut ran = has_ran_test.lock().unwrap();
            *ran = true;
        });
        let ran = has_ran.lock().unwrap();
        assert!(*ran);
    }
}
