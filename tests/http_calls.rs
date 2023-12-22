mod common;

use common::{test_with_containers, Params};
use reqwest::StatusCode;
use std::collections::HashMap;

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
fn hitting_api_with_ledgers_query() {
    let params = Params {
        quasar_port: 5444,
        playground_port: 7999,
        quasar_handle: "quasar_1".to_string(),
        stellar_port: 8002,
        stellar_handle: "stellar_1".to_string(),
        database_name: "quasar_dev1".to_string(),
    };

    test_with_containers(params.clone(), move || async move {
        let client = reqwest::Client::new();

        let mut query = HashMap::new();
        // probably need a custom value struct to add this
        // query.insert("variables", Some(&binding[..]));
        query.insert("operationName", None);

        let query_text = r#"
            {
                ledgers {
                    hash
                }
            }"#;

        query.insert("query", Some(query_text));

        let playground_url = &format!("http://127.0.0.1:{}", params.playground_port)[..];

        let req = client.post(playground_url).json(&query);

        let resp = req.send().await.unwrap();

        assert_eq!(resp.status(), reqwest::StatusCode::OK);

        let json_response_body = resp.text().await.unwrap();

        let ledgers_data: Data = serde_json::from_str(&json_response_body).unwrap();
        let ledgers_list: Vec<Ledger> = ledgers_data.data.ledgers;

        // this is directly correlated to the number of cycles
        assert_eq!(ledgers_list.len(), 22);
    });
}
