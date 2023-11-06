# Quasar

A Soroban indexer that offers a GraphQL API for ledgers, transactions, contracts, operations and events among others written in Rust.

Currently built to handle a subset of the Stellar dataset.

## Prerequisites

- You need a [running Stellar Core node](https://developers.stellar.org/docs/run-core-node/installation).

```
docker run --rm -it \
          -p "8000:8000" \
          -p "8001:5432" \
          --name stellar \
          stellar/quickstart:latest \
          --standalone \
          --enable-soroban-rpc
```

- You also need a running Postgres instance for persistence, eg:

```
docker run --name quasar -e POSTGRES_PASSWORD=quasar -e POSTGRES_DB=quasar_development  -p 5432:5432 -v postgres-data:/var/lib/postgresql/data -d postgres
```

## Setup

You will need a config file located in the path `config/config.toml`.

Here is an example:

```toml
quasar_database_url = "postgres://postgres:postgres@localhost:5432/quasar_development"
stellar_node_database_url = "postgres://stellar:<password>@localhost:8001/core"

[ingestion]
polling_interval = 5

[api]
host = "127.0.0.1"
port = 8000
depth_limit = 16
complexity_limit = 64

[metrics]
database_polling_interval = 5
```

## Getting started

Install `quasar` from the releases page. Here is an example in Linux:

```bash
  curl -L https://github.com/eigerco/quasar/releases/download/<version>/quasar_indexer-x86_64-unknown-linux-gnu.tar.gz > quasar.tar.gz
  tar -xf quasar.tar.gz
  ./quasar_indexer
```

GraphQL Playground will be available at `http://localhost:8000/`. Prometheus metrics at `http://localhost:8000/metrics`.

## Development

Clone the project

```bash
  git clone https://github.com/eigerco/quasar
```

Go to the project directory

```bash
  cd quasar
```

Run Tests

```bash
  cargo test
```

Start the server

```bash
  cargo run
```

## Overview of features

- Ingestion of:
  - ledgers
  - accounts
  - transactions
  - operations
  - contracts
  - events
- GraphQL:
  - Playground IDE with documentation
  - sorting
  - filtering
  - pagination
  - relationships
- Prometheus metrics

## Planned features

### Handling of the full Stellar dataset

Currently Quasar supports working with a limited subset of the Stellar dataset. We plan to support the full dataset, so that Quasar can run alongside a Full Validator, in the future. This will allow Quasar to be used as a data source for Stellar applications.

### GraphQL subscriptions

Subscriptions will allow clients to receive real-time updates from the indexer. For example a client can subscribe to a specific account and receive updates when the account's balance changes. This will be useful for building real-time applications.

### Processing of more data

The current version of Quasar supports only basic data types. Not all of their fields are ingested and processed. Some relationships between entities are also missing. We plan to add support for:

- more Stellar data types
- missing fields in existing data types
- more relationships between entities
- aggregated data

### More contract-specific GraphQL queries

We plan to add more GraphQL queries that are specific to Stellar contracts.

### Access control

We plan to add access control to the GraphQL API. This will allow owners of the server to restrict access to certain data.

## Contributing

Contributions are always welcome!

## Feedback

If you have any feedback, please reach out to us at hello@eiger.co
