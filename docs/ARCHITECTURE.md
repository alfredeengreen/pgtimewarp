# pgtimewarp architecture

pgtimewarp provides time travel for primary-key lookups on explicitly tracked tables, backed by WAL-derived row versions.

## Components

### Agent

The Rust agent consumes WAL changes via logical replication (wal2json for MVP). It:

- Creates and maintains a logical replication slot on the source database
- Consumes WAL changes and filters for tracked tables
- Transforms changes into row versions with validity intervals
- Writes row versions to the store database
- Maintains LSN to timestamp mappings for as-of query resolution
- Updates checkpoints for safe resume

### Store database

A dedicated PostgreSQL database containing:

- `nodes`: Agent instance registry
- `tracked_relations`: Explicit allowlist of tracked tables with primary key (node_id, schema_name, table_name)
- `row_versions`: Historical row states with LSN-based validity intervals
- `lsn_time_map`: Timestamp to LSN mappings
- `wal_checkpoints`: WAL consumption state
- `health_events`: Diagnostic events

### Extension

PostgreSQL C extension providing SQL functions:

- `pgtimewarp_asof_pk`: Get row version at timestamp
- `pgtimewarp_diff_pk`: Get change timeline between timestamps

The extension uses postgres_fdw to read from the store database. It resolves schema_name/table_name from regclass in the source database, then queries the store by (node_id, schema_name, table_name) to avoid trusting source OIDs.

### CLI

Rust CLI for management:

- `track`: Register a table for tracking
- `untrack`: Stop tracking a table
- `status`: List tracked tables and nodes
- `asof`: Query row at timestamp
- `diff`: Show change timeline
- `doctor`: Health diagnostics

## Data flow

1. Source database changes are written to WAL
2. Agent consumes WAL via logical replication slot
3. Agent filters changes for tracked tables
4. Agent transforms changes into row versions with validity intervals
5. Agent writes row versions to store database
6. Agent maintains LSN to timestamp mappings
7. Extension/CLI queries store database for time travel queries

## Database topology

By default, source_db (observed) and store_db (storage) are separate databases. This provides:

- Isolation of time travel data from production
- Independent scaling and backup strategies
- Security separation

The extension uses postgres_fdw to access the store database from the source database.

## LSN-based ordering

PostgreSQL WAL ordering is LSN-based, not timestamp-based. Timestamps can skew across transactions. pgtimewarp uses:

- `valid_from_lsn`/`valid_to_lsn`: Authoritative ordering
- `valid_from_ts`/`valid_to_ts`: Best-effort wall time
- `lsn_time_map`: Periodic mappings for timestamp → LSN conversion

As-of queries convert the requested timestamp to an LSN using lsn_time_map, then query by LSN ordering.

## Table identity

tracked_relations uses PRIMARY KEY (node_id, schema_name, table_name) for stability across dumps/restores. The relid OID is a nullable cache field updated by the agent at runtime.
