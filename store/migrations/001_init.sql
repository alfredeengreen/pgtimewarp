-- pgtimewarp store schema initialization
-- Creates core tables for time travel functionality

CREATE SCHEMA IF NOT EXISTS pgtimewarp;

-- Nodes registry: tracks agent instances and their health
CREATE TABLE pgtimewarp.nodes (
    node_id TEXT NOT NULL PRIMARY KEY,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    agent_version TEXT,
    meta JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Tracked relations: explicit allowlist of tables being tracked
-- Primary key is (node_id, schema_name, table_name) for stability across dumps/restores
-- relid is a nullable cache field updated by agent at runtime
CREATE TABLE pgtimewarp.tracked_relations (
    node_id TEXT NOT NULL,
    schema_name TEXT NOT NULL,
    table_name TEXT NOT NULL,
    relid OID NULL,
    pk_cols TEXT[] NOT NULL,
    pk_strategy SMALLINT NOT NULL,
    replica_identity_full BOOLEAN NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    retention_hours INT NOT NULL DEFAULT 24,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    meta JSONB NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (node_id, schema_name, table_name)
);

-- Row versions: stores historical row states with validity intervals
-- LSN-based ordering (valid_from_lsn/valid_to_lsn) for authoritative ordering
-- Timestamps (valid_from_ts/valid_to_ts) for best-effort wall time
CREATE TABLE pgtimewarp.row_versions (
    id BIGSERIAL PRIMARY KEY,
    node_id TEXT NOT NULL,
    relid OID NOT NULL,
    pk_hash BIGINT NOT NULL,
    valid_from_ts TIMESTAMPTZ NOT NULL,
    valid_to_ts TIMESTAMPTZ NULL,
    valid_from_lsn PG_LSN NOT NULL,
    valid_to_lsn PG_LSN NULL,
    op SMALLINT NOT NULL,
    row_data JSONB NULL,
    txid BIGINT NULL,
    confidence SMALLINT NOT NULL DEFAULT 2,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- LSN to timestamp mapping: enables timestamp -> LSN conversion for as-of queries
-- Agent writes periodic entries (every 1-5s) with last processed LSN and commit timestamp
CREATE TABLE pgtimewarp.lsn_time_map (
    node_id TEXT NOT NULL,
    ts TIMESTAMPTZ NOT NULL,
    lsn PG_LSN NOT NULL,
    PRIMARY KEY (node_id, ts)
);

-- WAL checkpoints: resume WAL consumption safely
CREATE TABLE pgtimewarp.wal_checkpoints (
    node_id TEXT NOT NULL PRIMARY KEY,
    slot_name TEXT NOT NULL,
    last_lsn PG_LSN NOT NULL,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    lag_bytes BIGINT NULL,
    meta JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Health events: diagnostics and monitoring
CREATE TABLE pgtimewarp.health_events (
    id BIGSERIAL PRIMARY KEY,
    node_id TEXT NOT NULL,
    ts TIMESTAMPTZ NOT NULL DEFAULT now(),
    kind TEXT NOT NULL,
    message TEXT NOT NULL,
    meta JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Status constants for tracked_relations.status
-- 0 = active
-- 1 = paused
-- 2 = needs_reinit

-- Operation constants for row_versions.op
-- 0 = insert
-- 1 = update
-- 2 = delete

-- Confidence constants for row_versions.confidence
-- 2 = high
-- 1 = medium
-- 0 = low
