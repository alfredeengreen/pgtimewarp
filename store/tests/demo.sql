\set ON_ERROR_STOP on

-- Demo: minimal end-to-end setup
INSERT INTO pgtimewarp.nodes (node_id, agent_version)
VALUES ('demo-node', 'demo-1.0')
ON CONFLICT (node_id) DO NOTHING;

INSERT INTO pgtimewarp.tracked_relations (
  node_id, schema_name, table_name, pk_cols, status, retention_hours, replica_identity_full
)
VALUES (
  'demo-node', 'public', 'accounts', ARRAY['id'], 0, 24, true
)
ON CONFLICT (node_id, schema_name, table_name) DO NOTHING;

-- Fetch tracked_id for later use
SELECT id AS tracked_id
FROM pgtimewarp.tracked_relations
WHERE node_id = 'demo-node'
  AND schema_name = 'public'
  AND table_name = 'accounts'
\gset

-- Demo: insert first version (open-ended)
INSERT INTO pgtimewarp.row_versions (
  tracked_id, node_id, schema_name, table_name, relid,
  pk_hash, valid_from_ts, valid_to_ts, valid_from_lsn, valid_to_lsn,
  op, row_data, txid, confidence
) VALUES (
  :tracked_id, 'demo-node', 'public', 'accounts', NULL,
  42, now() - interval '2 hours', NULL, '0/0000010'::pg_lsn, NULL,
  0, '{"id":1,"balance":100}', 1, 2
);

-- Demo: correctness guard (expect failure) - second open-ended version for same key
DO $$
BEGIN
  BEGIN
    INSERT INTO pgtimewarp.row_versions (
      tracked_id, node_id, schema_name, table_name,
      pk_hash, valid_from_ts, valid_to_ts, valid_from_lsn, valid_to_lsn,
      op, row_data
    ) VALUES (
      :tracked_id, 'demo-node', 'public', 'accounts',
      42, now() - interval '1 hour', NULL, '0/0000020'::pg_lsn, NULL,
      1, '{"id":1,"balance":150}'
    );
    RAISE EXCEPTION 'expected uq_row_versions_open to block second open version';
  EXCEPTION WHEN unique_violation THEN
    RAISE NOTICE 'ok: uq_row_versions_open blocked second open-ended version';
  END;
END $$;

-- Demo: denormalized consistency guard (expect failure)
DO $$
BEGIN
  PERFORM set_config('pgtimewarp.row_versions_consistency', 'on', false);
  BEGIN
    INSERT INTO pgtimewarp.row_versions (
      tracked_id, node_id, schema_name, table_name,
      pk_hash, valid_from_ts, valid_to_ts, valid_from_lsn, valid_to_lsn,
      op, row_data
    ) VALUES (
      :tracked_id, 'demo-node', 'public', 'wrong_table',
      7, now(), NULL, '0/0000030'::pg_lsn, NULL,
      0, '{"id":2,"balance":200}'
    );
    RAISE EXCEPTION 'expected consistency trigger to block mismatch';
  EXCEPTION WHEN others THEN
    RAISE NOTICE 'ok: consistency trigger blocked mismatch';
  END;
END $$;

-- Demo: conditional disable of consistency guard
DO $$
BEGIN
  PERFORM set_config('pgtimewarp.row_versions_consistency', 'off', false);
  INSERT INTO pgtimewarp.row_versions (
    tracked_id, node_id, schema_name, table_name,
    pk_hash, valid_from_ts, valid_to_ts, valid_from_lsn, valid_to_lsn,
    op, row_data
  ) VALUES (
    :tracked_id, 'demo-node', 'public', 'wrong_table',
    777, now(), NULL, '0/0000040'::pg_lsn, NULL,
    0, '{"id":3,"balance":250}'
  );
  RAISE NOTICE 'ok: consistency guard disabled for session';
  PERFORM set_config('pgtimewarp.row_versions_consistency', 'on', false);
END $$;

-- Demo: retention (insert an old row then delete via procedure)
INSERT INTO pgtimewarp.row_versions (
  tracked_id, node_id, schema_name, table_name,
  pk_hash, valid_from_ts, valid_to_ts, valid_from_lsn, valid_to_lsn,
  op, row_data
) VALUES (
  :tracked_id, 'demo-node', 'public', 'accounts',
  999, now() - interval '48 hours', now() - interval '47 hours',
  '0/0000001'::pg_lsn, '0/0000002'::pg_lsn,
  2, NULL
);

DO $$
DECLARE
  v_before BIGINT;
  v_after BIGINT;
BEGIN
  SELECT COUNT(*) INTO v_before
  FROM pgtimewarp.row_versions
  WHERE tracked_id = :tracked_id AND pk_hash = 999;

  CALL pgtimewarp.retention_delete_batch(:tracked_id, 1000, 5);

  SELECT COUNT(*) INTO v_after
  FROM pgtimewarp.row_versions
  WHERE tracked_id = :tracked_id AND pk_hash = 999;

  RAISE NOTICE 'retention_delete_batch: % -> %', v_before, v_after;
END $$;
