CREATE INDEX IF NOT EXISTS idx_tracked_relations_lookup
  ON pgtimewarp.tracked_relations (node_id, schema_name, table_name);

CREATE INDEX IF NOT EXISTS idx_tracked_relations_status
  ON pgtimewarp.tracked_relations (node_id, status);

-- Optional: accelerate "resolve relid -> tracked_id"
CREATE INDEX IF NOT EXISTS idx_tracked_relations_relid
  ON pgtimewarp.tracked_relations (node_id, relid)
  WHERE relid IS NOT NULL;
