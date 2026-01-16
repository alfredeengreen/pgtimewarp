CREATE TABLE IF NOT EXISTS pgtimewarp.health_events (
  id      BIGSERIAL PRIMARY KEY,
  node_id TEXT NOT NULL REFERENCES pgtimewarp.nodes(node_id) ON DELETE CASCADE,
  ts      TIMESTAMPTZ NOT NULL DEFAULT now(),
  kind    TEXT NOT NULL, -- heartbeat|degraded|schema_change|slot_lag|error|info
  message TEXT NOT NULL,
  meta    JSONB NOT NULL DEFAULT '{}'::jsonb
);
