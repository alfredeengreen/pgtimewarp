-- Test script for pgtimewarp store schema
-- Run this after applying all migrations to verify schema is correct

\echo 'Testing pgtimewarp schema...'

-- Test 1: Verify schema exists
SELECT CASE 
    WHEN EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'pgtimewarp')
    THEN 'PASS: Schema pgtimewarp exists'
    ELSE 'FAIL: Schema pgtimewarp missing'
END;

-- Test 2: Verify all tables exist
SELECT CASE 
    WHEN COUNT(*) = 6
    THEN 'PASS: All 6 tables exist'
    ELSE 'FAIL: Expected 6 tables, found ' || COUNT(*)::text
END
FROM pg_tables 
WHERE schemaname = 'pgtimewarp';

-- Test 3: Verify nodes table structure
SELECT CASE 
    WHEN COUNT(*) = 4
    THEN 'PASS: nodes table has correct columns'
    ELSE 'FAIL: nodes table column count mismatch'
END
FROM information_schema.columns
WHERE table_schema = 'pgtimewarp' AND table_name = 'nodes';

-- Test 4: Verify tracked_relations primary key
SELECT CASE 
    WHEN EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'tracked_relations_pkey' 
        AND contype = 'p'
    )
    THEN 'PASS: tracked_relations has primary key'
    ELSE 'FAIL: tracked_relations primary key missing'
END;

-- Test 5: Verify row_versions table exists and has required columns
SELECT CASE 
    WHEN COUNT(*) >= 10
    THEN 'PASS: row_versions has required columns'
    ELSE 'FAIL: row_versions missing columns'
END
FROM information_schema.columns
WHERE table_schema = 'pgtimewarp' AND table_name = 'row_versions';

-- Test 6: Verify lsn_time_map primary key
SELECT CASE 
    WHEN EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'lsn_time_map_pkey' 
        AND contype = 'p'
    )
    THEN 'PASS: lsn_time_map has primary key'
    ELSE 'FAIL: lsn_time_map primary key missing'
END;

-- Test 7: Insert and query test data
BEGIN;

-- Insert test node
INSERT INTO pgtimewarp.nodes (node_id, agent_version)
VALUES ('test-node', '0.1.0-test');

-- Insert test tracked relation
INSERT INTO pgtimewarp.tracked_relations 
    (node_id, schema_name, table_name, pk_cols, pk_strategy, replica_identity_full)
VALUES 
    ('test-node', 'public', 'test_table', ARRAY['id'], 0, true);

-- Insert test row version
INSERT INTO pgtimewarp.row_versions 
    (node_id, relid, pk_hash, valid_from_ts, valid_from_lsn, op, row_data)
VALUES 
    ('test-node', 12345, 67890, now(), '0/1000000', 0, '{"id": 1, "name": "test"}'::jsonb);

-- Insert test LSN time mapping
INSERT INTO pgtimewarp.lsn_time_map (node_id, ts, lsn)
VALUES ('test-node', now(), '0/1000000');

-- Verify inserts worked
SELECT CASE 
    WHEN COUNT(*) = 1
    THEN 'PASS: Test data inserted successfully'
    ELSE 'FAIL: Test data insertion failed'
END
FROM pgtimewarp.nodes WHERE node_id = 'test-node';

ROLLBACK;

\echo 'Schema tests complete!'
