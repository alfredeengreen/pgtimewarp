\set ON_ERROR_STOP on

-- Demo: show planner usage for as-of lookup by LSN
-- Assumes demo.sql has inserted at least one row.

SELECT id AS tracked_id
FROM pgtimewarp.tracked_relations
WHERE node_id = 'demo-node'
  AND schema_name = 'public'
  AND table_name = 'accounts'
\gset

EXPLAIN (ANALYZE, BUFFERS)
SELECT *
FROM pgtimewarp.row_versions
WHERE tracked_id = :tracked_id
  AND pk_hash = 42
  AND valid_from_lsn <= '0/0000100'::pg_lsn
ORDER BY valid_from_lsn DESC
LIMIT 1;
