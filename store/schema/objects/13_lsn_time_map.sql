-- Maps "approx commit time" -> "last processed commit LSN" for timestamp->LSN resolution.
-- Agent should insert at a fixed cadence (e.g., 1s or 5s) per node.
CREATE TABLE IF NOT EXISTS pgtimewarp.lsn_time_map (
  node_id TEXT NOT NULL REFERENCES pgtimewarp.nodes(node_id) ON DELETE CASCADE,
  ts      TIMESTAMPTZ NOT NULL,
  lsn     PG_LSN NOT NULL,
  PRIMARY KEY (node_id, ts)
);
