CREATE INDEX IF NOT EXISTS idx_health_events_node_kind_ts
  ON pgtimewarp.health_events (node_id, kind, ts DESC);

CREATE INDEX IF NOT EXISTS brin_health_events_ts
  ON pgtimewarp.health_events USING BRIN (ts);
