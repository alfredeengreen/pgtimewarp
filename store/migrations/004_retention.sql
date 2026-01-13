-- pgtimewarp retention functions
-- Implements throttled batch deletion for expired row versions

-- Retention function: delete expired row versions for a specific relation
-- Throttled to avoid long-running transactions
CREATE OR REPLACE FUNCTION pgtimewarp.retention_delete_batch(
    p_node_id TEXT,
    p_relid OID,
    p_batch_size INT DEFAULT 1000,
    p_max_duration INTERVAL DEFAULT '30 seconds'
) RETURNS TABLE(deleted_count BIGINT, duration INTERVAL) AS $$
DECLARE
    v_start_time TIMESTAMPTZ;
    v_deleted BIGINT := 0;
    v_retention_cutoff TIMESTAMPTZ;
    v_relation_retention_hours INT;
BEGIN
    v_start_time := clock_timestamp();
    
    -- Get retention window for this relation
    SELECT retention_hours INTO v_relation_retention_hours
    FROM pgtimewarp.tracked_relations
    WHERE node_id = p_node_id AND relid = p_relid;
    
    IF v_relation_retention_hours IS NULL THEN
        RETURN QUERY SELECT 0::BIGINT, clock_timestamp() - v_start_time;
        RETURN;
    END IF;
    
    v_retention_cutoff := now() - (v_relation_retention_hours || ' hours')::INTERVAL;
    
    -- Delete in batches until max duration or no more rows
    LOOP
        EXIT WHEN clock_timestamp() - v_start_time >= p_max_duration;
        
        WITH deleted AS (
            DELETE FROM pgtimewarp.row_versions
            WHERE node_id = p_node_id
              AND relid = p_relid
              AND valid_from_ts < v_retention_cutoff
              AND id IN (
                  SELECT id FROM pgtimewarp.row_versions
                  WHERE node_id = p_node_id
                    AND relid = p_relid
                    AND valid_from_ts < v_retention_cutoff
                  ORDER BY valid_from_ts
                  LIMIT p_batch_size
              )
            RETURNING id
        )
        SELECT COUNT(*) INTO v_deleted FROM deleted;
        
        EXIT WHEN v_deleted = 0;
        
        v_deleted := v_deleted + v_deleted;
        
        COMMIT;
    END LOOP;
    
    RETURN QUERY SELECT v_deleted, clock_timestamp() - v_start_time;
END;
$$ LANGUAGE plpgsql;

-- Retention function: process all active relations
CREATE OR REPLACE FUNCTION pgtimewarp.retention_run(
    p_node_id TEXT,
    p_batch_size INT DEFAULT 1000,
    p_max_duration INTERVAL DEFAULT '5 minutes'
) RETURNS TABLE(
    relid OID,
    deleted_count BIGINT,
    duration INTERVAL
) AS $$
DECLARE
    v_start_time TIMESTAMPTZ;
    v_relation_record RECORD;
    v_deleted BIGINT;
    v_duration INTERVAL;
BEGIN
    v_start_time := clock_timestamp();
    
    FOR v_relation_record IN
        SELECT DISTINCT relid
        FROM pgtimewarp.tracked_relations
        WHERE node_id = p_node_id
          AND status = 0
          AND relid IS NOT NULL
    LOOP
        EXIT WHEN clock_timestamp() - v_start_time >= p_max_duration;
        
        SELECT deleted_count, duration INTO v_deleted, v_duration
        FROM pgtimewarp.retention_delete_batch(
            p_node_id,
            v_relation_record.relid,
            p_batch_size,
            p_max_duration - (clock_timestamp() - v_start_time)
        );
        
        RETURN QUERY SELECT v_relation_record.relid, v_deleted, v_duration;
    END LOOP;
END;
$$ LANGUAGE plpgsql;
