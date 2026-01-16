CREATE TABLE IF NOT EXISTS pgtimewarp.nodes (
  node_id       TEXT PRIMARY KEY,
  first_seen    TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_seen     TIMESTAMPTZ NOT NULL DEFAULT now(),
  agent_version TEXT,
  meta          JSONB NOT NULL DEFAULT '{}'::jsonb
);
