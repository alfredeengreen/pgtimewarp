DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'pgtimewarp_writer') THEN
    CREATE ROLE pgtimewarp_writer;
  END IF;
END $$;

GRANT USAGE ON SCHEMA pgtimewarp TO pgtimewarp_writer;

GRANT SELECT, INSERT, UPDATE ON pgtimewarp.nodes TO pgtimewarp_writer;
GRANT SELECT, UPDATE           ON pgtimewarp.tracked_relations TO pgtimewarp_writer;
GRANT INSERT, UPDATE           ON pgtimewarp.row_versions TO pgtimewarp_writer;
GRANT INSERT                   ON pgtimewarp.lsn_time_map TO pgtimewarp_writer;
GRANT INSERT, UPDATE           ON pgtimewarp.wal_checkpoints TO pgtimewarp_writer;
GRANT INSERT                   ON pgtimewarp.health_events TO pgtimewarp_writer;

GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA pgtimewarp TO pgtimewarp_writer;
