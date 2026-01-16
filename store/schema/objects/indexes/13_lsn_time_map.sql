CREATE INDEX IF NOT EXISTS idx_lsn_time_map_ts_desc
  ON pgtimewarp.lsn_time_map (node_id, ts DESC);

CREATE INDEX IF NOT EXISTS idx_lsn_time_map_lsn_desc
  ON pgtimewarp.lsn_time_map (node_id, lsn DESC);
