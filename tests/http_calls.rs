mod common;

use common::{test_with_containers, Params};
use reqwest::StatusCode;
use std::collections::HashMap;

#[test]
fn hitting_localhost() {
    let params = Params::default();
    test_with_containers(params.clone(), move || async move {
        let playground_url = &format!("http://127.0.0.1:{}", params.playground_port)[..];

        let res = reqwest::get(playground_url).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    });
}

#[test]
fn query_ledgers_hashes() {
    let params = Params {
        quasar_port: 5444,
        playground_port: 7999,
        quasar_handle: "quasar_1".to_string(),
        stellar_port: 8002,
        stellar_handle: "stellar_1".to_string(),
        database_name: "quasar_dev1".to_string(),
    };

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct Ledger {
        hash: String,
    }

    #[derive(serde::Deserialize)]
    struct Ledgers {
        ledgers: Vec<Ledger>,
    }

    #[derive(serde::Deserialize)]
    struct Data {
        data: Ledgers,
    }

    let query_text = r#"
        {
            ledgers {
                hash
            }
        }"#;

    test_with_containers(params.clone(), move || async move {
        let client = reqwest::Client::new();

        let mut query = HashMap::new();
        // probably need a custom value struct to add this
        // query.insert("variables", Some(&binding[..]));
        query.insert("operationName", None);

        query.insert("query", Some(query_text));

        let playground_url = &format!("http://127.0.0.1:{}", params.playground_port)[..];

        let req = client.post(playground_url).json(&query);

        let resp = req.send().await.unwrap();

        assert_eq!(resp.status(), reqwest::StatusCode::OK);

        let json_response_body = resp.text().await.unwrap();

        let ledgers_data: Data = serde_json::from_str(&json_response_body).unwrap();
        let ledgers_list: Vec<Ledger> = ledgers_data.data.ledgers;

        // this is directly correlated to the number of cycles
        assert!(ledgers_list.len() > 5);
    });
}


// TODO to improve
#[test]
fn query_accounts_with_filters() {
    let params = Params {
        quasar_port: 5443,
        playground_port: 7998,
        quasar_handle: "quasar_2".to_string(),
        stellar_port: 8003,
        stellar_handle: "stellar_2".to_string(),
        database_name: "quasar_dev2".to_string(),
    };

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct Account {
        balance: u32,
    }

    #[derive(serde::Deserialize)]
    struct Accounts {
        accounts: Vec<Account>,
    }

    #[derive(serde::Deserialize)]
    struct Data {
        data: Accounts,
    }

    let query_text = r#"
        query {
            accounts(
                sort: {
                    balance: ASC
                }
                filter: {
                    balance: {
                        op: GREATER_THAN
                        value: 1
                    }
                    buyingLiabilities: {
                        op: GREATER_THAN
                        value: 1
                    }
                    sellingLiabilities: {
                        op: GREATER_THAN
                        value: 1
                    }
                    sequenceNumber: {
                        op: GREATER_THAN
                        value: 1
                    }
                }
                pagination: {
                    perPage: 2
                    page: 1
                }
            ) {
                id
            }
        }"#;

    test_with_containers(params.clone(), move || async move {
        let client = reqwest::Client::new();

        let mut query = HashMap::new();
        // probably need a custom value struct to add this
        // query.insert("variables", Some(&binding[..]));
        query.insert("operationName", None);

        query.insert("query", Some(query_text));

        let playground_url = &format!("http://127.0.0.1:{}", params.playground_port)[..];

        let req = client.post(playground_url).json(&query);

        let resp = req.send().await.unwrap();

        assert_eq!(resp.status(), reqwest::StatusCode::OK);

        let json_response_body = resp.text().await.unwrap();

        let accounts_data: Data = serde_json::from_str(&json_response_body).unwrap();
        let accounts_list: Vec<Account> = accounts_data.data.accounts;

        // this is directly correlated to the number of cycles
        assert!(accounts_list.len() == 0);
    });
}