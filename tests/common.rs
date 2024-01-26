use quasar_indexer::{
    configuration::setup_configuration,
    databases::{setup_quasar_database, setup_stellar_node_database},
    ingestion::ingest,
    server::serve,
};

use std::{future::Future};
use std::process::Command;
use std::sync::{Arc, Mutex};

use dockertest::waitfor::{MessageSource, MessageWait};
use dockertest::{DockerTest, Image, TestBodySpecification};
use tokio::time::{sleep, Duration};

use std::thread::sleep as sleep_sync;
use std::time::Duration as SyncDuration;

const PASSWORD_LINE: &str = "postgres password: ";

const PLAYGROUND_PORT: u16 = 8000;

const EXT_QUASAR_PORT: u32 = 5432;
const QUASAR_HANDLE: &str = "quasar";

const EXT_STELLAR_PORT: u32 = 9000;
const STELLAR_HANDLE: &str = "stellar";

const POSTGRES_PASSWORD: &str = "quasar";
const POSTGRES_DATABASE: &str = "quasar_development";

#[derive(Clone)]
pub struct Params {
    pub quasar_port: u32,
    pub stellar_port: u32,
    pub playground_port: u16,
    pub quasar_handle: String,
    pub stellar_handle: String,
    pub database_name: String,
}

impl Params {
    pub fn build(next_val: u32) -> Self {
        Self {
            quasar_port: EXT_QUASAR_PORT + next_val,
            quasar_handle: format!("{}_{}", QUASAR_HANDLE, 1 + next_val),
            playground_port: PLAYGROUND_PORT + next_val as u16,
            stellar_port: EXT_STELLAR_PORT + next_val,
            stellar_handle: format!("{}_{}", STELLAR_HANDLE, 10 + next_val),
            database_name: format!("{}_{}", POSTGRES_DATABASE, 100 + next_val),
        }
    }
}

fn quasar_database_spec(ext_port: u32, db_name: &str, handle: &str) -> TestBodySpecification {
    let exit_wait = MessageWait {
        message: "database system is ready to accept connections".to_string(),
        source: MessageSource::Stderr,
        timeout: 5,
    };

    let mut quasar_proc = TestBodySpecification::with_repository("postgres")
        .set_wait_for(Box::new(exit_wait))
        .set_handle(handle);

    quasar_proc.modify_port_map(5432, ext_port);
    quasar_proc.modify_named_volume("postgres-data", "/var/lib/postgresql/");
    quasar_proc.modify_env("POSTGRES_PASSWORD", POSTGRES_PASSWORD);
    quasar_proc.modify_env("POSTGRES_DB", db_name);

    quasar_proc
}

fn stellar_node_spec(ext_port: u32, handle: &str) -> TestBodySpecification {
    let exit_wait = MessageWait {
        // it has to arrive to this point otherwise weird
        // SSH messages will appear
        message: "INFO spawned: 'nginx'".to_string(),
        source: MessageSource::Stdout,
        timeout: 15,
    };

    let image = Image::with_repository("stellar/quickstart").tag("soroban-dev");

    let mut soroban_proc = TestBodySpecification::with_image(image)
        .set_wait_for(Box::new(exit_wait))
        .set_handle(handle);

    soroban_proc.modify_port_map(5432, ext_port);
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

    let output = std::str::from_utf8(&out.stdout).unwrap();

    let remind = output.split(PASSWORD_LINE).collect::<Vec<&str>>();
    let password = remind.get(1).unwrap().split("\n").collect::<Vec<&str>>();

    password.get(0).unwrap().to_string()
}

pub fn test_with_containers<Fut>(
    params: Params,
    running_test: impl FnOnce() -> Fut + Send + 'static,
) where
    Fut: Future<Output = ()> + Send + 'static,
{
    // Define our test instance, will pull images from dockerhub.
    let mut test = DockerTest::new();

    test.provide_container(quasar_database_spec(
        params.quasar_port,
        &params.database_name,
        &params.quasar_handle,
    ))
    .provide_container(stellar_node_spec(
        params.stellar_port,
        &params.stellar_handle,
    ));

    // Run a command.
    println!("containers provided..");

    let has_ran: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let has_ran_test = has_ran.clone();

    test.run(|ops| async move {
        // A handle to operate on the Container.
        let _ = ops.handle(&params.quasar_handle);
        let stellar_container = ops.handle(&params.stellar_handle);

        let mut configuration = setup_configuration(None);

        println!("config setup..");
        let quasar_conn_str = format!(
            "postgres://postgres:quasar@localhost:{}/{}",
            params.quasar_port, params.database_name
        );
        configuration.quasar_database_url = Some(quasar_conn_str);
        let quasar_database = setup_quasar_database(&configuration).await;
        println!("connected to quasar database..");

        let password = get_password_from_logs(stellar_container.name());
        let conn_str = format!(
            "postgres://stellar:{}@localhost:{}/core",
            password, params.stellar_port
        );
        println!("stellar conn_str: {}", conn_str);

        configuration.stellar_node_database_url = Some(conn_str);

        let node_database = setup_stellar_node_database(&configuration).await;
        println!("connected to stellar!");

        configuration.api.port = params.playground_port;

        // Start the HTTP server, including GraphQL API
        serve(&configuration.api, quasar_database.clone(), None).await;

        println!("started the http server");

        tokio::spawn(async move {
            // Start the ingestion loop
            ingest(
                node_database,
                quasar_database,
                configuration.ingestion,
                None,
            )
            .await;
        });

        let wait_time = configuration.ingestion.polling_interval * 5;

        println!(
            "Going to sleep for {} seconds to give time to ingest data.",
            wait_time
        );

        // giving time to digest ...
        sleep(Duration::from_secs(wait_time)).await;

        println!("after ingest!");

        running_test().await;

        let mut ran = has_ran_test.lock().unwrap();
        *ran = true;
    });
    let ran = has_ran.lock().unwrap();
    assert!(*ran);
    // wait a moment after switching off
    sleep_sync(SyncDuration::from_secs(1));
}
