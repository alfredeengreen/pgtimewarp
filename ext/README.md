# pgtimewarp extension

PostgreSQL extension providing SQL functions for time travel queries.

## Installation

1. Build the extension:

```bash
make
sudo make install
```

2. Create the extension in your database:

```sql
CREATE EXTENSION pgtimewarp;
```

## Functions

### pgtimewarp_asof_pk

Get a row version at a specific timestamp.

```sql
SELECT pgtimewarp_asof_pk('public.orders', '{"id": 123}'::jsonb, '2026-01-13 09:41:00'::timestamptz);
```

### pgtimewarp_diff_pk

Get the change timeline for a row between two timestamps.

```sql
SELECT * FROM pgtimewarp_diff_pk(
    'public.orders',
    '{"id": 123}'::jsonb,
    '2026-01-13 09:00:00'::timestamptz,
    '2026-01-13 10:00:00'::timestamptz
);
```

## FDW setup

The extension requires postgres_fdw to access the store database. Set up foreign tables:

```sql
CREATE SERVER pgtimewarp_store
FOREIGN DATA WRAPPER postgres_fdw
OPTIONS (host 'localhost', dbname 'pgtimewarp');

CREATE USER MAPPING FOR CURRENT_USER
SERVER pgtimewarp_store
OPTIONS (user 'pgtimewarp_reader', password 'password');

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
