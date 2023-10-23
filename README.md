# Quasar

A Soroban indexer that offers a GraphQL API for ledgers, transactions, contracts, operations and events among others written in Rust.

## Prerequisites

- You need a [running Stellar Core node](https://developers.stellar.org/docs/run-core-node/installation).
- You also need a running Postgres instance for persistence.

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
  curl -L https://github.com/eigerco/quasar/releases/download/v0.1.0/quasar > quasar
  ./quasar
```

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

## Contributing

Contributions are always welcome!

## Feedback

If you have any feedback, please reach out to us at hello@eiger.co
