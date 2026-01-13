# pgtimewarp agent

Rust WAL consumer that captures row versions from PostgreSQL logical replication.

## Configuration

Create a `config.yaml` file:

```yaml
node_id: "prod-eu-1"
agent_version: "0.1.0"
source:
  dsn: "postgres://user:password@localhost/sourcedb"
  slot_name: "pgtimewarp_prod_eu_1"
  plugin: "wal2json"
  wal2json_options:
    include_lsn: true
    include_timestamp: true
    include_old: true
store:
  dsn: "postgres://user:password@localhost/pgtimewarp"
intervals:
  refresh_tracked_s: 30
  retention_s: 300
limits:
  batch_size: 500
  max_queue: 100000
```

## Running

```bash
cargo run -- --config config.yaml
```

Or build and run:

```bash
cargo build --release
./target/release/pgtimewarp-agent --config config.yaml
```

## Requirements

- PostgreSQL source database with logical replication enabled
- wal2json extension installed on source database
- Store database with pgtimewarp schema initialized
