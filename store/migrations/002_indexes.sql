-- pgtimewarp index strategy
-- Optimized for as-of lookups, retention scans, and timestamp -> LSN conversion

-- Tracked relations indexes
CREATE INDEX IF NOT EXISTS idx_tracked_relations_schema_table 
    ON pgtimewarp.tracked_relations (schema_name, table_name);

CREATE INDEX IF NOT EXISTS idx_tracked_relations_status 
    ON pgtimewarp.tracked_relations (status);

-- Row versions indexes for as-of lookups (LSN-based ordering)
CREATE INDEX IF NOT EXISTS idx_row_versions_asof 
    ON pgtimewarp.row_versions (node_id, relid, pk_hash, valid_from_lsn DESC);

-- Row versions indexes for retention operations
CREATE INDEX IF NOT EXISTS idx_row_versions_retention_ts 
    ON pgtimewarp.row_versions (node_id, relid, valid_from_ts);

-- LSN time map index for timestamp -> LSN conversion
CREATE INDEX IF NOT EXISTS idx_lsn_time_map_lookup 
    ON pgtimewarp.lsn_time_map (node_id, ts DESC);

-- Health events indexes for time-series queries
CREATE INDEX IF NOT EXISTS idx_health_events_node_kind_ts 
    ON pgtimewarp.health_events (node_id, kind, ts DESC);

-- BRIN indexes for large-scale retention scans
CREATE INDEX IF NOT EXISTS idx_row_versions_retention_brin 
    ON pgtimewarp.row_versions USING BRIN (valid_from_ts);

CREATE INDEX IF NOT EXISTS idx_health_events_brin 
    ON pgtimewarp.health_events USING BRIN (node_id, ts);
