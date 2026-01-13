# Testing Guide

This document describes how to test the pgtimewarp components.

## Prerequisites

- PostgreSQL 12+ installed
- Rust toolchain installed
- PostgreSQL development headers (for extension)

## Agent Tests

Run agent unit tests:

```bash
cd agent
cargo test
```

Run agent build check:

```bash
cd agent
cargo check
```

## CLI Tests

Run CLI unit tests:

```bash
cd cli
cargo test
```

Build the CLI:

```bash
cd cli
cargo build
```

## Store Schema Tests

1. Create a test database:

```bash
createdb pgtimewarp_test
```

2. Apply migrations:

```bash
psql -d pgtimewarp_test -f store/migrations/001_init.sql
psql -d pgtimewarp_test -f store/migrations/002_indexes.sql
psql -d pgtimewarp_test -f store/migrations/003_roles_grants.sql
psql -d pgtimewarp_test -f store/migrations/004_retention.sql
```

3. Run schema tests:

```bash
psql -d pgtimewarp_test -f store/test_schema.sql
```

4. Clean up:

```bash
dropdb pgtimewarp_test
```

## Extension Tests

1. Build the extension:

```bash
cd ext
make
```

2. Install the extension (requires sudo):

```bash
cd ext
sudo make install
```

3. Test in PostgreSQL:

```sql
CREATE DATABASE pgtimewarp_test;
\c pgtimewarp_test

-- Apply store migrations first
\i store/migrations/001_init.sql
\i store/migrations/002_indexes.sql

-- Create extension
CREATE EXTENSION pgtimewarp;

-- Verify functions exist
\df pgtimewarp_asof_pk
\df pgtimewarp_diff_pk
```

## Integration Tests

For end-to-end testing, you'll need:

1. A source PostgreSQL database with:
   - `wal_level = logical`
   - `wal2json` extension installed
   - A test table with `REPLICA IDENTITY FULL`

2. A store PostgreSQL database with migrations applied

3. Agent configuration file

Example integration test:

```bash
# 1. Set up source database
createdb pgtimewarp_source_test
psql -d pgtimewarp_source_test -c "CREATE EXTENSION wal2json;"
psql -d pgtimewarp_source_test -c "CREATE TABLE test_orders (id SERIAL PRIMARY KEY, total DECIMAL, status TEXT);"
psql -d pgtimewarp_source_test -c "ALTER TABLE test_orders REPLICA IDENTITY FULL;"

# 2. Set up store database
createdb pgtimewarp_store_test
psql -d pgtimewarp_store_test -f store/migrations/001_init.sql
psql -d pgtimewarp_store_test -f store/migrations/002_indexes.sql
psql -d pgtimewarp_store_test -f store/migrations/003_roles_grants.sql
psql -d pgtimewarp_store_test -f store/migrations/004_retention.sql

# 3. Create test config
cp agent/config.yaml.example agent/config.test.yaml
# Edit config.test.yaml with test database DSNs

# 4. Track the test table
./cli/target/debug/pgtimewarp-cli track public.test_orders \
  --pk id \
  --retention 24 \
  --node test-node \
  --store-dsn "postgres://postgres@localhost/pgtimewarp_store_test"

# 5. Start the agent
./agent/target/debug/pgtimewarp-agent --config agent/config.test.yaml &

# 6. Make changes to source table
psql -d pgtimewarp_source_test -c "INSERT INTO test_orders (total, status) VALUES (99.99, 'pending');"
psql -d pgtimewarp_source_test -c "UPDATE test_orders SET status = 'confirmed' WHERE id = 1;"

# 7. Query historical data
./cli/target/debug/pgtimewarp-cli asof public.test_orders \
  --pk id=1 \
  --at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --node test-node \
  --store-dsn "postgres://postgres@localhost/pgtimewarp_store_test"

# 8. Clean up
killall pgtimewarp-agent
dropdb pgtimewarp_source_test
dropdb pgtimewarp_store_test
```

## CI/CD

GitHub Actions will automatically run tests on push. See `.github/workflows/ci.yml` for details.

## Troubleshooting Tests

### Agent tests fail with connection errors
- Ensure PostgreSQL is not required for unit tests
- Mock database connections in tests

### Extension build fails
- Install PostgreSQL development headers: `apt-get install postgresql-server-dev-XX`
- Ensure `pg_config` is in PATH

### Integration tests fail
- Check PostgreSQL logs for replication errors
- Verify `wal_level = logical` in postgresql.conf
- Ensure `wal2json` extension is installed
