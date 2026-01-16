# pgtimewarp store schema

The store database contains the time travel history for tracked tables. This schema is separate from the source database being observed.

## Setup

1. Create a dedicated PostgreSQL database for the store:

```sql
CREATE DATABASE pgtimewarp;
```

2. Connect to the store database and run migrations in order:

```bash
psql -d pgtimewarp -f migrations/001_init.sql
psql -d pgtimewarp -f migrations/002_indexes.sql
psql -d pgtimewarp -f migrations/003_triggers.sql
psql -d pgtimewarp -f migrations/004_roles_grants.sql
psql -d pgtimewarp -f migrations/005_retention.sql
```

Or run all migrations:

```bash
for f in migrations/*.sql; do
    psql -d pgtimewarp -f "$f"
done
```

## Schema overview

- `nodes`: Agent instance registry and health tracking
- `tracked_relations`: Explicit allowlist of tracked tables with stable `id` and unique (node_id, schema_name, table_name)
- `row_versions`: Historical row states keyed by `tracked_id` with LSN-based validity intervals
- `lsn_time_map`: Timestamp to LSN mappings for as-of query resolution
- `wal_checkpoints`: WAL consumption state for safe resume
- `health_events`: Diagnostic events and monitoring data

## Roles

- `pgtimewarp_writer`: Agent writes row versions, checkpoints, health events
- `pgtimewarp_reader`: CLI and extension read-only access
- `pgtimewarp_admin`: CLI manages tracking (track/untrack operations)

## Retention

Retention is managed per relation via the `retention_hours` column in `tracked_relations`. The `retention_run` procedure processes all active relations, and `retention_delete_batch` handles throttled deletion for a specific relation.

Run retention manually:

```sql
CALL pgtimewarp.retention_run('node-id', 1000, 300);
```

## Maintenance

The schema is designed for high write throughput. Consider partitioning `row_versions` by time for very large deployments. BRIN indexes are provided for efficient retention scans.

## Demo tests

Run a small demo script against the store database:

```bash
psql -d pgtimewarp -f tests/demo.sql
```

Optional: show index usage for an as-of lookup:

```bash
psql -d pgtimewarp -f tests/explain_demo.sql
```
