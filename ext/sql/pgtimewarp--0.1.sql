CREATE OR REPLACE FUNCTION pgtimewarp_asof_pk(
    qualified regclass,
    pk jsonb,
    as_of timestamptz,
    node_id text DEFAULT 'default'
) RETURNS jsonb AS $$
DECLARE
    v_schema_name text;
    v_table_name text;
    v_pk_hash bigint;
    v_as_of_lsn pg_lsn;
    v_row_data jsonb;
    v_valid_from_ts timestamptz;
    v_valid_from_lsn pg_lsn;
BEGIN
    SELECT n.nspname, c.relname
    INTO v_schema_name, v_table_name
    FROM pg_class c
    JOIN pg_namespace n ON c.relnamespace = n.oid
    WHERE c.oid = qualified;
    
    IF v_schema_name IS NULL OR v_table_name IS NULL THEN
        RAISE EXCEPTION 'relation % not found', qualified;
    END IF;
    
    SELECT lsn INTO v_as_of_lsn
    FROM pgtimewarp.lsn_time_map
    WHERE pgtimewarp.lsn_time_map.node_id = node_id
      AND ts <= as_of
    ORDER BY ts DESC
    LIMIT 1;
    
    IF v_as_of_lsn IS NULL THEN
        RETURN jsonb_build_object('error', 'no LSN mapping found for timestamp');
    END IF;
    
    SELECT rv.row_data, rv.valid_from_ts, rv.valid_from_lsn::text
    INTO v_row_data, v_valid_from_ts, v_valid_from_lsn
    FROM pgtimewarp.row_versions rv
    JOIN pgtimewarp.tracked_relations tr
      ON rv.node_id = tr.node_id AND rv.relid = tr.relid
    WHERE tr.node_id = node_id
      AND tr.schema_name = v_schema_name
      AND tr.table_name = v_table_name
      AND rv.valid_from_lsn <= v_as_of_lsn
      AND (rv.valid_to_lsn IS NULL OR rv.valid_to_lsn > v_as_of_lsn)
    ORDER BY rv.valid_from_lsn DESC
    LIMIT 1;
    
    IF v_row_data IS NULL THEN
        RETURN jsonb_build_object('error', 'no row found at specified time');
    END IF;
    
    RETURN jsonb_build_object(
        'row', v_row_data,
        'effective_as_of_ts', v_valid_from_ts,
        'effective_as_of_lsn', v_valid_from_lsn
    );
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION pgtimewarp_diff_pk(
    qualified regclass,
    pk jsonb,
    from_ts timestamptz,
    to_ts timestamptz,
    node_id text DEFAULT 'default'
) RETURNS SETOF jsonb AS $$
DECLARE
    v_schema_name text;
    v_table_name text;
    v_from_lsn pg_lsn;
    v_to_lsn pg_lsn;
    v_row_data jsonb;
    v_valid_from_ts timestamptz;
    v_valid_from_lsn pg_lsn;
    v_op smallint;
BEGIN
    SELECT n.nspname, c.relname
    INTO v_schema_name, v_table_name
    FROM pg_class c
    JOIN pg_namespace n ON c.relnamespace = n.oid
    WHERE c.oid = qualified;
    
    IF v_schema_name IS NULL OR v_table_name IS NULL THEN
        RAISE EXCEPTION 'relation % not found', qualified;
    END IF;
    
    SELECT lsn INTO v_from_lsn
    FROM pgtimewarp.lsn_time_map
    WHERE pgtimewarp.lsn_time_map.node_id = node_id
      AND ts <= from_ts
    ORDER BY ts DESC
    LIMIT 1;
    
    SELECT lsn INTO v_to_lsn
    FROM pgtimewarp.lsn_time_map
    WHERE pgtimewarp.lsn_time_map.node_id = node_id
      AND ts <= to_ts
    ORDER BY ts DESC
    LIMIT 1;
    
    IF v_from_lsn IS NULL OR v_to_lsn IS NULL THEN
        RETURN;
    END IF;
    
    FOR v_row_data, v_valid_from_ts, v_valid_from_lsn, v_op IN
        SELECT rv.row_data, rv.valid_from_ts, rv.valid_from_lsn::text, rv.op
        FROM pgtimewarp.row_versions rv
        JOIN pgtimewarp.tracked_relations tr
          ON rv.node_id = tr.node_id AND rv.relid = tr.relid
        WHERE tr.node_id = node_id
          AND tr.schema_name = v_schema_name
          AND tr.table_name = v_table_name
          AND rv.valid_from_lsn >= v_from_lsn
          AND rv.valid_from_lsn <= v_to_lsn
        ORDER BY rv.valid_from_lsn
    LOOP
        RETURN NEXT jsonb_build_object(
            'op', CASE v_op WHEN 0 THEN 'insert' WHEN 1 THEN 'update' WHEN 2 THEN 'delete' ELSE 'unknown' END,
            'row', v_row_data,
            'valid_from_ts', v_valid_from_ts,
            'valid_from_lsn', v_valid_from_lsn
        );
    END LOOP;
    
    RETURN;
END;
$$ LANGUAGE plpgsql;
