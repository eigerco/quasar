use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Configuration {
    pub database_url: Option<String>,
    pub stellar_node_database_url: Option<String>,
}
