mod common;

use common::{test_with_containers, Params};
use quasar_indexer::{ingestion::sleep, configuration::Ingestion};
use reqwest::StatusCode;
use std::collections::HashMap;
use stellar_sdk::Keypair;

#[test]
fn hitting_localhost() {
    let params = Params::build(0);
    test_with_containers(params.clone(), move || async move {
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

        let json_data: serde_json::Value = serde_json::from_str(&json_response_body).unwrap();
        assert!(json_data.is_object());
        let data = json_data.as_object().unwrap().get("data").unwrap();
        let ledgers = data.get("ledgers").unwrap();
        assert!(ledgers.is_array());
        assert!(ledgers.as_array().unwrap().len() > 5);
    });
}

// TODO to improve
#[test]
fn query_accounts_with_filters() {
    let params = Params::build(2);
    
    test_with_containers(params.clone(), move || async move {
        let client = reqwest::Client::new();
        
        let public_key = create_account(&params, &client).await;

        println!("pk {}", public_key);

        let query_text = format!(r#"
            query {{
                account(
                    id: "{}"
                ) {{
                    id
                }}
            }}"#, public_key);

        let ingestion = Ingestion{polling_interval: 5};
        sleep(&ingestion).await;

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
        let json_data: serde_json::Value = serde_json::from_str(&json_response_body).unwrap();
        assert!(json_data.is_object());
        let data = json_data.as_object().unwrap().get("data").unwrap();
        println!("data {}", json_data);
        let accounts = data.get("account").unwrap();
        assert!(accounts.is_array());
        println!("acc: {}", accounts.as_array().unwrap().len());
        for account in accounts.as_array().unwrap() {
            print!("ac: {}", account);
        }
        assert_eq!(accounts.as_array().unwrap().len(), 1);
    });
}

async fn create_account(params: &Params, client: &reqwest::Client) -> String {
        
    let key_pair = Keypair::random().unwrap();

    let req = client.get(format!("http://127.0.0.1:{}/friendbot?addr={}", params.stellar_node_port, key_pair.public_key()));
    let resp = req.send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    key_pair.public_key()
}
