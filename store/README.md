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
psql -d pgtimewarp -f migrations/003_roles_grants.sql
psql -d pgtimewarp -f migrations/004_retention.sql
```

Or run all migrations:

```bash
for f in migrations/*.sql; do
    psql -d pgtimewarp -f "$f"
done
```

## Schema overview

- `nodes`: Agent instance registry and health tracking
- `tracked_relations`: Explicit allowlist of tracked tables with primary key (node_id, schema_name, table_name)
- `row_versions`: Historical row states with LSN-based validity intervals
- `lsn_time_map`: Timestamp to LSN mappings for as-of query resolution
- `wal_checkpoints`: WAL consumption state for safe resume
- `health_events`: Diagnostic events and monitoring data

## Roles

- `pgtimewarp_writer`: Agent writes row versions, checkpoints, health events
- `pgtimewarp_reader`: CLI and extension read-only access
- `pgtimewarp_admin`: CLI manages tracking (track/untrack operations)

## Retention

Retention is managed per relation via the `retention_hours` column in `tracked_relations`. The `retention_run()` function processes all active relations, and `retention_delete_batch()` handles throttled deletion for a specific relation.

Run retention manually:

```sql
SELECT * FROM pgtimewarp.retention_run('node-id', 1000, '5 minutes'::interval);
```

## Maintenance

The schema is designed for high write throughput. Consider partitioning `row_versions` by time for very large deployments. BRIN indexes are provided for efficient retention scans.
