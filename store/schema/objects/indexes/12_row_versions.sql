-- Fast as-of lookup by LSN (authoritative)
CREATE INDEX IF NOT EXISTS idx_row_versions_asof_lsn
  ON pgtimewarp.row_versions (tracked_id, pk_hash, valid_from_lsn DESC);

-- Time-based filtering / retention
CREATE INDEX IF NOT EXISTS idx_row_versions_retention
  ON pgtimewarp.row_versions (tracked_id, valid_from_ts);

CREATE INDEX IF NOT EXISTS brin_row_versions_valid_from_ts
  ON pgtimewarp.row_versions USING BRIN (valid_from_ts);

-- Common analytic query: all changes for a row over time
CREATE INDEX IF NOT EXISTS idx_row_versions_timeline
  ON pgtimewarp.row_versions (tracked_id, pk_hash, valid_from_ts DESC);
