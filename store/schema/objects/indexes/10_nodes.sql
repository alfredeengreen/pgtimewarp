CREATE INDEX IF NOT EXISTS idx_nodes_last_seen ON pgtimewarp.nodes (last_seen DESC);
