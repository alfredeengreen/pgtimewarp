# pgtimewarp limitations (MVP)

## Scope

- Primary key-based lookups only. No arbitrary SQL time travel or joins.
- Tables must be explicitly tracked via CLI.
- Requires REPLICA IDENTITY FULL for tracked tables to capture full row state on UPDATE/DELETE.

## Data types

- No TOAST support in MVP. Large values may not be captured correctly.
- JSONB storage for row data. No msgpack encoding in MVP.

## Performance

- Retention is throttled batch deletion. Very high write rates may require partitioning.
- LSN to timestamp mapping is sampled (every 1-5 seconds). Sub-second precision may be limited.

## Operational

- wal2json extension required on source database for MVP.
- Separate store database required (cannot use same database as source).
- Agent must maintain connection to both source and store databases.

## Correctness

- LSN-based ordering is authoritative. Timestamps are best-effort and may skew.
- Table identity uses (node_id, schema_name, table_name). OID changes after dumps/restores are handled, but agent must refresh relid cache.

## Security

- Store database contains full row content. Encryption at rest recommended.
- postgres_fdw requires network access between source and store databases.
