# pgtimewarp

PostgreSQL time travel for primary-key lookups on explicitly tracked tables, backed by WAL-derived row versions.

## Overview

pgtimewarp provides time travel functionality for PostgreSQL databases. It captures row versions from WAL (Write-Ahead Log) via logical replication and enables queries like "show row X as-of time T" and "show change timeline for row X".

The system consists of four components:

- **Agent**: Rust WAL consumer that captures row versions from logical replication
- **Store**: PostgreSQL database schema for storing historical row versions with LSN-based ordering
- **Extension**: PostgreSQL C extension providing SQL functions for time travel queries
- **CLI**: Rust command-line tool for management and queries

## Prerequisites

- PostgreSQL 12+ with logical replication enabled
- wal2json extension installed on source database
- Rust toolchain (for building agent and CLI)
- PostgreSQL development headers (for building extension)

## Installation

### 1. Build components

Build the agent and CLI:

```bash
cd agent
cargo build --release

cd ../cli
cargo build --release
```

Build the extension:

```bash
cd ext
make
sudo make install
```

### 2. Set up source database

Enable logical replication in `postgresql.conf`:

```conf
wal_level = logical
max_replication_slots = 10
max_wal_senders = 10
```

Restart PostgreSQL, then create a replication role:

```sql
CREATE ROLE pgtimewarp_replication WITH REPLICATION LOGIN PASSWORD 'secure_password';
GRANT CONNECT ON DATABASE your_database TO pgtimewarp_replication;
```

Install wal2json extension:

```bash
# Debian/Ubuntu
sudo apt-get install postgresql-15-wal2json

# Or build from source
git clone https://github.com/eulerto/wal2json.git
cd wal2json
make && sudo make install
```

Enable the extension in your database:

```sql
CREATE EXTENSION wal2json;
```

### 3. Set up store database

Create the store database:

```sql
CREATE DATABASE pgtimewarp;
```

Run migrations:

```bash
psql -d pgtimewarp -f store/migrations/001_init.sql
psql -d pgtimewarp -f store/migrations/002_indexes.sql
psql -d pgtimewarp -f store/migrations/003_roles_grants.sql
psql -d pgtimewarp -f store/migrations/004_retention.sql
```

Create roles:

```sql
CREATE ROLE pgtimewarp_writer WITH LOGIN PASSWORD 'secure_password';
GRANT pgtimewarp_writer TO pgtimewarp_writer;

CREATE ROLE pgtimewarp_reader WITH LOGIN PASSWORD 'secure_password';
GRANT pgtimewarp_reader TO pgtimewarp_reader;

CREATE ROLE pgtimewarp_admin WITH LOGIN PASSWORD 'secure_password';
GRANT pgtimewarp_admin TO pgtimewarp_admin;
```

### 4. Configure agent

Create `agent/config.yaml`:

```yaml
node_id: "prod-eu-1"
agent_version: "0.1.0"
source:
  dsn: "postgres://pgtimewarp_replication:secure_password@localhost/your_database"
  slot_name: "pgtimewarp_prod_eu_1"
  plugin: "wal2json"
  wal2json_options:
    include_lsn: true
    include_timestamp: true
    include_typmod: false
    include_pk: true
    pretty_print: false
    write_in_chunks: false
    include_old: true
store:
  dsn: "postgres://pgtimewarp_writer:secure_password@localhost/pgtimewarp"
intervals:
  refresh_tracked_s: 30
  retention_s: 300
limits:
  batch_size: 500
  max_queue: 100000
```

### 5. Start agent

```bash
cd agent
./target/release/pgtimewarp-agent --config config.yaml
```

The agent will create the replication slot automatically and start consuming WAL changes.

## Usage

### Track a table

Before tracking, ensure the table has REPLICA IDENTITY FULL:

```sql
ALTER TABLE public.orders REPLICA IDENTITY FULL;
```

Track the table using the CLI:

```bash
pgtimewarp track public.orders \
  --pk id \
  --retention 168 \
  --node prod-eu-1 \
  --store-dsn "postgres://pgtimewarp_admin:secure_password@localhost/pgtimewarp"
```

This tracks the `orders` table with:
- Primary key: `id`
- Retention: 168 hours (7 days)
- Node: `prod-eu-1`

For composite primary keys:

```bash
pgtimewarp track public.order_items \
  --pk order_id,item_id \
  --retention 72 \
  --node prod-eu-1 \
  --store-dsn "postgres://pgtimewarp_admin:secure_password@localhost/pgtimewarp"
```

### Query row at specific time

Using the CLI:

```bash
pgtimewarp asof public.orders \
  --pk id=12345 \
  --at "2026-01-13T14:30:00Z" \
  --node prod-eu-1 \
  --store-dsn "postgres://pgtimewarp_reader:secure_password@localhost/pgtimewarp"
```

Output:

```json
{
  "row": {
    "id": 12345,
    "customer_id": 789,
    "total": 299.99,
    "status": "pending",
    "created_at": "2026-01-13T10:00:00Z"
  },
  "effective_as_of_ts": "2026-01-13T14:28:15Z",
  "effective_as_of_lsn": "0/1234567"
}
```

Using SQL (requires extension setup):

```sql
SELECT pgtimewarp_asof_pk(
  'public.orders',
  '{"id": 12345}'::jsonb,
  '2026-01-13 14:30:00'::timestamptz,
  'prod-eu-1'
);
```

### View change timeline

Get all changes to a row between two timestamps:

```bash
pgtimewarp diff public.orders \
  --pk id=12345 \
  --from "2026-01-13T00:00:00Z" \
  --to "2026-01-13T23:59:59Z" \
  --node prod-eu-1 \
  --store-dsn "postgres://pgtimewarp_reader:secure_password@localhost/pgtimewarp"
```

Output:

```json
{
  "versions": [
    {
      "op": "insert",
      "row": {
        "id": 12345,
        "customer_id": 789,
        "total": 299.99,
        "status": "pending"
      },
      "valid_from_ts": "2026-01-13T10:00:00Z",
      "valid_from_lsn": "0/1000000"
    },
    {
      "op": "update",
      "row": {
        "id": 12345,
        "customer_id": 789,
        "total": 299.99,
        "status": "confirmed"
      },
      "valid_from_ts": "2026-01-13T14:30:00Z",
      "valid_from_lsn": "0/1234567"
    },
    {
      "op": "update",
      "row": {
        "id": 12345,
        "customer_id": 789,
        "total": 299.99,
        "status": "shipped"
      },
      "valid_from_ts": "2026-01-13T18:45:00Z",
      "valid_from_lsn": "0/1500000"
    }
  ]
}
```

Using SQL:

```sql
SELECT * FROM pgtimewarp_diff_pk(
  'public.orders',
  '{"id": 12345}'::jsonb,
  '2026-01-13 00:00:00'::timestamptz,
  '2026-01-13 23:59:59'::timestamptz,
  'prod-eu-1'
);
```

### Check status

List all tracked tables:

```bash
pgtimewarp status \
  --store-dsn "postgres://pgtimewarp_reader:secure_password@localhost/pgtimewarp"
```

Output:

```
Nodes:
  prod-eu-1 - last seen: 2026-01-13T15:30:00Z - version: 0.1.0

Tracked tables:
  prod-eu-1: public.orders - active
  prod-eu-1: public.order_items - active
  prod-eu-1: public.customers - active
```

Check status for a specific node:

```bash
pgtimewarp status --node prod-eu-1 \
  --store-dsn "postgres://pgtimewarp_reader:secure_password@localhost/pgtimewarp"
```

### Health diagnostics

Check agent health:

```bash
pgtimewarp doctor \
  --store-dsn "postgres://pgtimewarp_reader:secure_password@localhost/pgtimewarp"
```

Check health for a specific node:

```bash
pgtimewarp doctor --node prod-eu-1 \
  --store-dsn "postgres://pgtimewarp_reader:secure_password@localhost/pgtimewarp"
```

### Untrack a table

Stop tracking a table:

```bash
pgtimewarp untrack public.orders \
  --node prod-eu-1 \
  --store-dsn "postgres://pgtimewarp_admin:secure_password@localhost/pgtimewarp"
```

## Common workflows

### Audit trail

Track sensitive tables to maintain an audit trail:

```bash
# Track user accounts
pgtimewarp track public.users \
  --pk id \
  --retention 8760 \
  --node prod-eu-1

# Track financial transactions
pgtimewarp track public.transactions \
  --pk id \
  --retention 8760 \
  --node prod-eu-1
```

Query historical state:

```bash
# Check user account state at time of incident
pgtimewarp asof public.users \
  --pk id=42 \
  --at "2026-01-10T12:00:00Z" \
  --node prod-eu-1
```

### Debugging data issues

View change timeline to understand when and how data changed:

```bash
# See all changes to an order
pgtimewarp diff public.orders \
  --pk id=12345 \
  --from "2026-01-01T00:00:00Z" \
  --to "2026-01-13T23:59:59Z" \
  --node prod-eu-1
```

### Point-in-time recovery

Recover row state at a specific point in time:

```bash
# Get customer data before a problematic update
pgtimewarp asof public.customers \
  --pk id=789 \
  --at "2026-01-12T10:00:00Z" \
  --node prod-eu-1
```

## Configuration

### Environment variables

Set store DSN via environment variable:

```bash
export PGTIMEWARP_STORE_DSN="postgres://pgtimewarp_reader:secure_password@localhost/pgtimewarp"
pgtimewarp status
```

### Agent configuration options

- `node_id`: Unique identifier for this agent instance
- `agent_version`: Version string for tracking
- `source.dsn`: Source database connection string
- `source.slot_name`: Logical replication slot name
- `source.plugin`: Replication plugin (wal2json for MVP)
- `source.wal2json_options`: wal2json-specific options
- `store.dsn`: Store database connection string
- `intervals.refresh_tracked_s`: How often to refresh tracked relations cache
- `intervals.retention_s`: How often to run retention jobs
- `limits.batch_size`: Batch size for row version writes
- `limits.max_queue`: Maximum queue size before backpressure

## Extension setup

For SQL-based queries, set up the extension:

1. Build and install (see Installation section)

2. Create extension in source database:

```sql
CREATE EXTENSION pgtimewarp;
```

3. Set up postgres_fdw to access store database:

```sql
CREATE SERVER pgtimewarp_store
FOREIGN DATA WRAPPER postgres_fdw
OPTIONS (host 'localhost', dbname 'pgtimewarp', port '5432');

CREATE USER MAPPING FOR CURRENT_USER
SERVER pgtimewarp_store
OPTIONS (user 'pgtimewarp_reader', password 'secure_password');

CREATE FOREIGN TABLE pgtimewarp.tracked_relations (
    node_id text,
    schema_name text,
    table_name text,
    relid oid,
    pk_cols text[],
    pk_strategy smallint,
    replica_identity_full boolean,
    status smallint,
    retention_hours integer,
    created_at timestamptz,
    updated_at timestamptz,
    meta jsonb
) SERVER pgtimewarp_store
OPTIONS (schema_name 'pgtimewarp', table_name 'tracked_relations');

CREATE FOREIGN TABLE pgtimewarp.row_versions (
    id bigint,
    node_id text,
    relid oid,
    pk_hash bigint,
    valid_from_ts timestamptz,
    valid_to_ts timestamptz,
    valid_from_lsn pg_lsn,
    valid_to_lsn pg_lsn,
    op smallint,
    row_data jsonb,
    txid bigint,
    confidence smallint,
    created_at timestamptz
) SERVER pgtimewarp_store
OPTIONS (schema_name 'pgtimewarp', table_name 'row_versions');

CREATE FOREIGN TABLE pgtimewarp.lsn_time_map (
    node_id text,
    ts timestamptz,
    lsn pg_lsn
) SERVER pgtimewarp_store
OPTIONS (schema_name 'pgtimewarp', table_name 'lsn_time_map');
```

See `ext/README.md` for complete FDW setup instructions.

## Retention

Retention is managed per table via the `retention_hours` setting. Run retention manually:

```sql
SELECT * FROM pgtimewarp.retention_run('prod-eu-1', 1000, '5 minutes'::interval);
```

This deletes row versions older than the retention window for each tracked table.

## Troubleshooting

### Agent not capturing changes

1. Check replication slot exists:

```sql
SELECT * FROM pg_replication_slots WHERE slot_name = 'pgtimewarp_prod_eu_1';
```

2. Verify table is tracked:

```bash
pgtimewarp status --node prod-eu-1
```

3. Check table has REPLICA IDENTITY FULL:

```sql
SELECT relreplident FROM pg_class WHERE relname = 'orders';
-- Should return 'f' for FULL
```

### No data in queries

1. Verify agent is running and processing changes
2. Check LSN time map has entries:

```sql
SELECT COUNT(*) FROM pgtimewarp.lsn_time_map WHERE node_id = 'prod-eu-1';
```

3. Verify row versions exist:

```sql
SELECT COUNT(*) FROM pgtimewarp.row_versions WHERE node_id = 'prod-eu-1';
```

### Extension queries fail

1. Verify FDW is set up correctly
2. Check foreign tables are accessible:

```sql
SELECT * FROM pgtimewarp.tracked_relations LIMIT 1;
```

3. Verify node_id matches between source and store

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - System design and data flow
- [Deployment](docs/DEPLOYMENT.md) - Detailed setup and configuration
- [Limitations](docs/LIMITATIONS.md) - MVP constraints and known issues
- [Security](docs/SECURITY.md) - Security considerations and best practices

## License

Apache 2.0 - see [LICENSE](LICENSE)
