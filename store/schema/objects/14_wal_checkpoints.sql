CREATE TABLE IF NOT EXISTS pgtimewarp.wal_checkpoints (
  node_id   TEXT PRIMARY KEY REFERENCES pgtimewarp.nodes(node_id) ON DELETE CASCADE,
  slot_name TEXT NOT NULL,
  last_lsn  PG_LSN NOT NULL,
  last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
  lag_bytes BIGINT NULL,
  meta      JSONB NOT NULL DEFAULT '{}'::jsonb
);
