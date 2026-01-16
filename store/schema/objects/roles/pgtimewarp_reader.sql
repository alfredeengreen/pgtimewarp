DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'pgtimewarp_reader') THEN
    CREATE ROLE pgtimewarp_reader;
  END IF;
END $$;

GRANT USAGE ON SCHEMA pgtimewarp TO pgtimewarp_reader;

GRANT SELECT ON pgtimewarp.nodes TO pgtimewarp_reader;
GRANT SELECT ON pgtimewarp.tracked_relations TO pgtimewarp_reader;
GRANT SELECT ON pgtimewarp.row_versions TO pgtimewarp_reader;
GRANT SELECT ON pgtimewarp.lsn_time_map TO pgtimewarp_reader;
GRANT SELECT ON pgtimewarp.wal_checkpoints TO pgtimewarp_reader;
GRANT SELECT ON pgtimewarp.health_events TO pgtimewarp_reader;
