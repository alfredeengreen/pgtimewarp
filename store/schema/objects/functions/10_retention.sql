-- Deletes in small chunks to avoid long locks and bloat spikes.
-- Uses tracked_relations.retention_hours and operates per tracked_id.
-- NOTE: transaction control is allowed in PROCEDURE (CALL), not FUNCTION.

CREATE OR REPLACE PROCEDURE pgtimewarp.retention_delete_batch(
  p_tracked_id   BIGINT,
  p_batch_size   INT DEFAULT 5000,
  p_max_seconds  INT DEFAULT 30
)
LANGUAGE plpgsql
AS $$
DECLARE
  v_start       TIMESTAMPTZ := clock_timestamp();
  v_cutoff      TIMESTAMPTZ;
  v_hours       INT;
  v_deleted     BIGINT;
BEGIN
  SELECT retention_hours INTO v_hours
  FROM pgtimewarp.tracked_relations
  WHERE id = p_tracked_id;

  IF v_hours IS NULL THEN
    RETURN;
  END IF;

  v_cutoff := now() - make_interval(hours => v_hours);

  LOOP
    EXIT WHEN EXTRACT(EPOCH FROM (clock_timestamp() - v_start)) >= p_max_seconds;

    WITH del AS (
      DELETE FROM pgtimewarp.row_versions
      WHERE tracked_id = p_tracked_id
        AND valid_from_ts < v_cutoff
        AND id IN (
          SELECT id
          FROM pgtimewarp.row_versions
          WHERE tracked_id = p_tracked_id
            AND valid_from_ts < v_cutoff
          ORDER BY valid_from_ts
          LIMIT p_batch_size
        )
      RETURNING 1
    )
    SELECT COUNT(*) INTO v_deleted FROM del;

    EXIT WHEN v_deleted = 0;

    COMMIT;  -- allowed in PROCEDURE
    START TRANSACTION;
  END LOOP;
END $$;

CREATE OR REPLACE PROCEDURE pgtimewarp.retention_run(
  p_node_id      TEXT,
  p_batch_size   INT DEFAULT 5000,
  p_max_seconds  INT DEFAULT 300
)
LANGUAGE plpgsql
AS $$
DECLARE
  v_start TIMESTAMPTZ := clock_timestamp();
  r RECORD;
BEGIN
  -- Ensure we can use COMMIT/START TRANSACTION in this procedure call context
  -- Caller should CALL this in its own session.
  FOR r IN
    SELECT id
    FROM pgtimewarp.tracked_relations
    WHERE node_id = p_node_id
      AND status = 0
    ORDER BY id
  LOOP
    EXIT WHEN EXTRACT(EPOCH FROM (clock_timestamp() - v_start)) >= p_max_seconds;

    CALL pgtimewarp.retention_delete_batch(
      r.id,
      p_batch_size,
      GREATEST(1, p_max_seconds - EXTRACT(EPOCH FROM (clock_timestamp() - v_start))::INT)
    );
  END LOOP;
END $$;
