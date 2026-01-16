CREATE TABLE IF NOT EXISTS pgtimewarp.tracked_relations (
  id                    BIGSERIAL PRIMARY KEY,

  node_id               TEXT NOT NULL REFERENCES pgtimewarp.nodes(node_id) ON DELETE CASCADE,
  schema_name           TEXT NOT NULL,
  table_name            TEXT NOT NULL,

  -- Cache only: resolved dynamically by agent. Not stable across rebuilds.
  relid                 OID NULL,

  -- Primary key identity in source table
  pk_cols               TEXT[] NOT NULL CHECK (array_length(pk_cols, 1) >= 1),

  -- Operational controls
  status                SMALLINT NOT NULL DEFAULT 0, -- 0=active, 1=paused, 2=needs_reinit
  retention_hours       INT NOT NULL DEFAULT 24 CHECK (retention_hours > 0),
  replica_identity_full BOOLEAN NOT NULL DEFAULT false,

  created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  meta                  JSONB NOT NULL DEFAULT '{}'::jsonb,

  UNIQUE (node_id, schema_name, table_name)
);

-- If you choose to rely on relid for any hot-path queries, keep it unique when present.
CREATE UNIQUE INDEX IF NOT EXISTS uq_tracked_relations_node_relid
  ON pgtimewarp.tracked_relations (node_id, relid)
  WHERE relid IS NOT NULL;
