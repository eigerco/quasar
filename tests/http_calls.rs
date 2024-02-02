mod common;

use common::{test_with_containers, Params};
use quasar_indexer::{
    configuration::{Configuration, Ingestion},
    ingestion::sleep,
};
use reqwest::{Client, StatusCode};
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
fn query_ledgers() {
    let params = Params::build(1);

    let query_text = r#"
        {
            ledgers {
                hash
            }
        }"#;

    test_with_containers(
        params.clone(),
        move |configuration: Configuration| async move {
            let client = Client::new();

            create_account(&params, &client).await;
            create_account(&params, &client).await;
            let ledger_hashes = get_ledger_hashes(&params, &client).await;
            wait_for_ingestion(configuration).await;

            let mut query = HashMap::new();
            query.insert("operationName", None);
            query.insert("query", Some(query_text));

            let data = submit_query(&client, &query, &params).await;
            let ledgers = data.get("ledgers").unwrap();
            assert!(ledgers.is_array());
            let ledgers_array = ledgers.as_array().unwrap();

            //find ledger hashes matching the ones returned from chain
            //should be the same amount
            let filtered_ledgers = ledgers_array
                .into_iter()
                .map(|x| {
                    x.as_object()
                        .unwrap()
                        .get("hash")
                        .unwrap()
                        .as_str()
                        .unwrap()
                })
                .filter(|id| ledger_hashes.contains(&id.to_string()))
                .collect::<Vec<&str>>();
            assert_eq!(filtered_ledgers.len(), ledger_hashes.len());
        },
    );
}

#[test]
fn query_transactions() {
    let params = Params::build(2);

    let query_text = r#"
        {
            transactions {
                id
            }
        }"#;

    test_with_containers(
        params.clone(),
        move |configuration: Configuration| async move {
            let client = Client::new();

            let id_1 = create_account(&params, &client).await;
            let id_2 = create_account(&params, &client).await;
            let mut transaction_ids = get_account_transaction_ids(&params, &client, &id_1).await;
            transaction_ids.append(&mut get_account_transaction_ids(&params, &client, &id_2).await);
            wait_for_ingestion(configuration).await;

            let mut query = HashMap::new();
            query.insert("operationName", None);
            query.insert("query", Some(query_text));

            let data = submit_query(&client, &query, &params).await;
            let transactions = data.get("transactions").unwrap();
            assert!(transactions.is_array());

            let transactions_array = transactions.as_array().unwrap();
            //find transaction ids matching the ones returned by chain
            //should be the same amount
            let filtered_transactions = transactions_array
                .into_iter()
                .map(|x| x.as_object().unwrap().get("id").unwrap().as_str().unwrap())
                .filter(|id| transaction_ids.contains(&id.to_string()))
                .collect::<Vec<&str>>();
            assert_eq!(filtered_transactions.len(), transaction_ids.len());
        },
    );
}

#[test]
fn query_created_account() {
    let params = Params::build(3);

    test_with_containers(
        params.clone(),
        move |configuration: Configuration| async move {
            let client = Client::new();

            let public_key = create_account(&params, &client).await;
            wait_for_ingestion(configuration).await;

            let query_text = format!(
                r#"
                query {{
                    account(
                        id: "{}"
                    ) {{
                        id
                    }}
                }}"#,
                public_key
            );

            let mut query = HashMap::new();
            query.insert("operationName", None);
            query.insert("query", Some(&query_text[..]));

            let data = submit_query(&client, &query, &params).await;
            let account = data.get("account").unwrap();
            assert!(account.is_object());
            let account_id = account
                .as_object()
                .unwrap()
                .get("id")
                .unwrap()
                .as_str()
                .unwrap();
            assert_eq!(account_id, public_key);
        },
    );
}

#[test]
fn query_accounts_with_filters() {
    let params = Params::build(4);

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
                id
            }
        }"#;

    test_with_containers(
        params.clone(),
        move |configuration: Configuration| async move {
            let client = Client::new();

            //create two accounts
            create_account(&params, &client).await;
            create_account(&params, &client).await;
            wait_for_ingestion(configuration).await;

            let mut query = HashMap::new();
            query.insert("operationName", None);
            query.insert("query", Some(query_text));

            let data = submit_query(&client, &query, &params).await;
            let accounts = data.get("accounts").unwrap();
            assert!(accounts.is_array());
            //two accounts should be returned
            assert_eq!(accounts.as_array().unwrap().len(), 2);
        },
    );
}

async fn submit_query(
    client: &Client,
    query: &HashMap<&str, Option<&str>>,
    params: &Params,
) -> Value {
    let playground_url = &format!("http://127.0.0.1:{}", params.playground_port)[..];

    let req = client.post(playground_url).json(&query);
    let resp = req.send().await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let json_response_body = resp.text().await.unwrap();
    let json_data: serde_json::Value = serde_json::from_str(&json_response_body).unwrap();
    assert!(json_data.is_object());
    json_data.as_object().unwrap().get("data").unwrap().clone()
}

async fn create_account(params: &Params, client: &Client) -> String {
    let key_pair = Keypair::random().unwrap();

    let url = format!(
        "http://127.0.0.1:{}/friendbot?addr={}",
        params.stellar_node_port,
        key_pair.public_key()
    );
    let req = client.get(url);
    let resp = req.send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    key_pair.public_key()
}

async fn get_account_transaction_ids(
    params: &Params,
    client: &Client,
    account_id: &String,
) -> Vec<String> {
    let url = format!(
        "http://127.0.0.1:{}/accounts/{}/transactions",
        params.stellar_node_port, account_id
    );
    let req = client.get(url);
    let resp = req.send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json_response_body = resp.text().await.unwrap();
    let json_data: serde_json::Value = serde_json::from_str(&json_response_body).unwrap();
    assert!(json_data.is_object());
    let embedded_data = json_data.as_object().unwrap().get("_embedded").unwrap();
    let records_data = embedded_data
        .as_object()
        .unwrap()
        .get("records")
        .unwrap()
        .as_array()
        .unwrap();

    let transaction_ids = records_data
        .into_iter()
        .map(|record| {
            record
                .as_object()
                .unwrap()
                .get("id")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<String>>();

    transaction_ids
}

async fn get_ledger_hashes(params: &Params, client: &Client) -> Vec<String> {
    let url = format!("http://127.0.0.1:{}/ledgers", params.stellar_node_port);
    let req = client.get(url);
    let resp = req.send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json_response_body = resp.text().await.unwrap();
    let json_data: serde_json::Value = serde_json::from_str(&json_response_body).unwrap();
    assert!(json_data.is_object());
    let embedded_data = json_data.as_object().unwrap().get("_embedded").unwrap();
    let records_data = embedded_data
        .as_object()
        .unwrap()
        .get("records")
        .unwrap()
        .as_array()
        .unwrap();

    let ledger_hashes = records_data
        .into_iter()
        .map(|record| {
            record
                .as_object()
                .unwrap()
                .get("hash")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<String>>();

    ledger_hashes
}

async fn wait_for_ingestion(configuration: Configuration) {
    let ingestion = Ingestion {
        polling_interval: configuration.ingestion.polling_interval,
    };
    sleep(&ingestion).await;
}
