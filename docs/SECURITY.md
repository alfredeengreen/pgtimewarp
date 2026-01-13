# pgtimewarp security

## Data storage

The store database contains full row content from tracked tables. Consider:

- Encryption at rest for the store database
- Network encryption (TLS) for database connections
- Access controls via PostgreSQL roles

## Role-based access

Three roles provide separation of concerns:

- `pgtimewarp_writer`: Agent writes row versions, checkpoints, health events
- `pgtimewarp_reader`: CLI and extension read-only access
- `pgtimewarp_admin`: CLI manages tracking (track/untrack operations)

Grant minimal required permissions to each component.

## Network security

postgres_fdw requires network access between source and store databases. Use:

- Firewall rules to restrict access
- TLS connections
- VPN or private networks for production

## Table allowlist

Only explicitly tracked tables are captured. Use the CLI allowlist feature to restrict which tables can be tracked:

```yaml
privacy:
  allow_tables:
    - "public.orders"
    - "public.customers"
```

## Replication slot security

Logical replication slots require replication privilege. Use a dedicated role with minimal permissions:

```sql
CREATE ROLE pgtimewarp_replication WITH REPLICATION LOGIN PASSWORD 'strong-password';
```

## Source database access

The agent requires:

- Replication privilege for logical replication slot
- SELECT on tracked tables (via replication protocol)
- No direct write access to source database

## Store database access

The agent requires INSERT/UPDATE on:

- pgtimewarp.row_versions
- pgtimewarp.lsn_time_map
- pgtimewarp.wal_checkpoints
- pgtimewarp.health_events
- pgtimewarp.nodes

The extension requires SELECT on:

- pgtimewarp.tracked_relations
- pgtimewarp.row_versions
- pgtimewarp.lsn_time_map

The CLI requires:

- SELECT for read operations
- INSERT/UPDATE/DELETE on pgtimewarp.tracked_relations for track/untrack
