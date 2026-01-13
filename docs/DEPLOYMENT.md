# pgtimewarp deployment

## Source database requirements

1. Enable logical replication:

```sql
wal_level = logical
max_replication_slots >= 1
max_wal_senders >= 1
```

2. Create a role with replication privilege:

```sql
CREATE ROLE pgtimewarp_replication WITH REPLICATION LOGIN PASSWORD 'password';
```

3. Install wal2json extension:

```bash
# On Debian/Ubuntu
sudo apt-get install postgresql-XX-wal2json

# Or build from source
git clone https://github.com/eulerto/wal2json.git
cd wal2json
make && sudo make install
```

Then in the database:

```sql
CREATE EXTENSION wal2json;
```

## Store database setup

1. Create the store database:

```sql
CREATE DATABASE pgtimewarp;
```

2. Run migrations:

```bash
psql -d pgtimewarp -f store/migrations/001_init.sql
psql -d pgtimewarp -f store/migrations/002_indexes.sql
psql -d pgtimewarp -f store/migrations/003_roles_grants.sql
psql -d pgtimewarp -f store/migrations/004_retention.sql
```

3. Create roles for components:

```sql
CREATE ROLE pgtimewarp_writer WITH LOGIN PASSWORD 'password';
GRANT pgtimewarp_writer TO pgtimewarp_writer;

CREATE ROLE pgtimewarp_reader WITH LOGIN PASSWORD 'password';
GRANT pgtimewarp_reader TO pgtimewarp_reader;

CREATE ROLE pgtimewarp_admin WITH LOGIN PASSWORD 'password';
GRANT pgtimewarp_admin TO pgtimewarp_admin;
```

## Agent configuration

Create `config.yaml`:

```yaml
node_id: "prod-eu-1"
agent_version: "0.1.0"
source:
  dsn: "postgres://pgtimewarp_replication:password@localhost/sourcedb"
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
  dsn: "postgres://pgtimewarp_writer:password@localhost/pgtimewarp"
intervals:
  refresh_tracked_s: 30
  retention_s: 300
limits:
  batch_size: 500
  max_queue: 100000
```

Run the agent:

```bash
cargo run --bin pgtimewarp-agent -- --config config.yaml
```

## Extension setup

1. Build and install:

```bash
cd ext
make
sudo make install
```

2. Create extension in source database:

```sql
CREATE EXTENSION pgtimewarp;
```

3. Set up postgres_fdw (see ext/README.md for details)

## Track a table

Use the CLI to track a table:

```bash
pgtimewarp track public.orders --pk id --retention 24 --node prod-eu-1 \
  --store-dsn "postgres://pgtimewarp_admin:password@localhost/pgtimewarp"
```

The table must have REPLICA IDENTITY FULL:

```sql
ALTER TABLE public.orders REPLICA IDENTITY FULL;
```

## Query time travel

From SQL:

```sql
SELECT pgtimewarp_asof_pk('public.orders', '{"id": 123}'::jsonb, '2026-01-13 09:41:00'::timestamptz);
```

From CLI:

```bash
pgtimewarp asof public.orders --pk id=123 --at "2026-01-13T09:41:00Z" --node prod-eu-1
```
