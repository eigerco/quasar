mod common;

use common::{test_with_containers, Params};
use quasar_indexer::{ingestion::sleep, configuration::{Ingestion, Configuration}};
use reqwest::{StatusCode, Client};
use serde_json::Value;
use std::collections::HashMap;
use stellar_sdk::Keypair;

#[test]
fn hitting_localhost() {
    let params = Params::build(0);
    test_with_containers(params.clone(), move |_: Configuration| async move {
        let playground_url = &format!("http://127.0.0.1:{}", params.playground_port)[..];
        
        let res = reqwest::get(playground_url).await.unwrap();
        
        assert_eq!(res.status(), StatusCode::OK);
    });
}

#[test]
fn query_ledgers_hashes() {
    let params = Params::build(1);

    let query_text = r#"
        {
            ledgers {
                hash
            }
        }"#;

    test_with_containers(params.clone(), move |configuration: Configuration| async move {
        let client = Client::new();

        let mut query = HashMap::new();
        query.insert("operationName", None);
        query.insert("query", Some(query_text));

        let data = submit_query(&client, &query, &params).await;
        let ledgers = data.get("ledgers").unwrap();
        assert!(ledgers.is_array());
        assert!(ledgers.as_array().unwrap().len() > 5);
        let ledgers_count = ledgers.as_array().unwrap().len();

        //add two accounts which should create two transactions
        create_account(&params, &client).await;
        create_account(&params, &client).await;
        wait_for_ingestion(configuration).await;

        let data = submit_query(&client, &query, &params).await;
        let ledgers = data.get("ledgers").unwrap();

        assert!(ledgers.as_array().unwrap().len() - ledgers_count >= 2);
    });
}

#[test]
fn query_created_account() {
    let params = Params::build(2);
    
    test_with_containers(params.clone(), move |configuration: Configuration| async move {
        let client = Client::new();
        
        let public_key = create_account(&params, &client).await;
        wait_for_ingestion(configuration).await;

        let query_text = format!(r#"
            query {{
                account(
                    id: "{}"
                ) {{
                    id
                }}
            }}"#, public_key);

        let mut query = HashMap::new();
        query.insert("operationName", None);
        query.insert("query", Some(&query_text[..]));

        let data = submit_query(&client, &query, &params).await;
        let account = data.get("account").unwrap();
        assert!(account.is_object());
        let account_id = account.as_object().unwrap().get("id").unwrap().as_str().unwrap();
        assert_eq!(account_id, public_key);
    });
}

#[test]
fn query_accounts_with_filters() {
    let params = Params::build(3);

    let query_text = r#"
        query {
            accounts(
                sort: {
                    balance: ASC
                }
                filter: {
                    balance: {
                        op: EQUAL
                        value: 100000000000
                    }
                }
                pagination: {
                    perPage: 20
                    page: 1
                }
            ) {
                id, sequenceNumber
            }
        }"#;

    test_with_containers(params.clone(), move |configuration: Configuration| async move {
        let client = Client::new();

        //create two accounts
        create_account(&params, &client).await;
        create_account(&params, &client).await;
        wait_for_ingestion(configuration).await;

        let mut query = HashMap::new();
        query.insert("operationName", None);
        query.insert("query", Some(query_text));

        let data = submit_query(&client, &query, &params).await;
        println!("accs: {}", data);
        let accounts = data.get("accounts").unwrap();
        assert!(accounts.is_array());
        //two accounts should be returned
        assert_eq!(accounts.as_array().unwrap().len(), 2);
    });
}

async fn submit_query(client: &Client, query: &HashMap<&str, Option<&str>>, params: &Params) -> Value {
    let playground_url = &format!("http://127.0.0.1:{}", params.playground_port)[..];

    let req = client.post(playground_url).json(&query);
    let resp = req.send().await.unwrap();

    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    let json_response_body = resp.text().await.unwrap();
    let json_data: serde_json::Value = serde_json::from_str(&json_response_body).unwrap();
    assert!(json_data.is_object());
    json_data.as_object().unwrap().get("data").unwrap().clone()
}

async fn create_account(params: &Params, client: &Client) -> String {
    let key_pair = Keypair::random().unwrap();

    let req = client.get(format!("http://127.0.0.1:{}/friendbot?addr={}", params.stellar_node_port, key_pair.public_key()));
    let resp = req.send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    key_pair.public_key()
}

async fn wait_for_ingestion(configuration: Configuration) {
    let ingestion = Ingestion{polling_interval: configuration.ingestion.polling_interval};
    sleep(&ingestion).await;
}

