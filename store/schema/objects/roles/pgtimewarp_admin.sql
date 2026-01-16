DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'pgtimewarp_admin') THEN
    CREATE ROLE pgtimewarp_admin;
  END IF;
END $$;

GRANT USAGE ON SCHEMA pgtimewarp TO pgtimewarp_admin;

GRANT SELECT, INSERT, UPDATE, DELETE ON pgtimewarp.tracked_relations TO pgtimewarp_admin;

-- Admin typically reads everything; writes are reserved for writer
GRANT SELECT ON pgtimewarp.nodes TO pgtimewarp_admin;
GRANT SELECT ON pgtimewarp.row_versions TO pgtimewarp_admin;
GRANT SELECT ON pgtimewarp.lsn_time_map TO pgtimewarp_admin;
GRANT SELECT ON pgtimewarp.wal_checkpoints TO pgtimewarp_admin;
GRANT SELECT ON pgtimewarp.health_events TO pgtimewarp_admin;

GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA pgtimewarp TO pgtimewarp_admin;
