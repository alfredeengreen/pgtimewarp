-- pgtimewarp roles and grants
-- Defines role-based access control for different components

-- Writer role: agent writes row versions, checkpoints, health events
CREATE ROLE pgtimewarp_writer;

GRANT USAGE ON SCHEMA pgtimewarp TO pgtimewarp_writer;

GRANT INSERT, UPDATE ON pgtimewarp.nodes TO pgtimewarp_writer;
GRANT SELECT, UPDATE ON pgtimewarp.tracked_relations TO pgtimewarp_writer;
GRANT INSERT, UPDATE ON pgtimewarp.row_versions TO pgtimewarp_writer;
GRANT INSERT ON pgtimewarp.lsn_time_map TO pgtimewarp_writer;
GRANT INSERT, UPDATE ON pgtimewarp.wal_checkpoints TO pgtimewarp_writer;
GRANT INSERT ON pgtimewarp.health_events TO pgtimewarp_writer;

GRANT USAGE, SELECT ON SEQUENCE pgtimewarp.row_versions_id_seq TO pgtimewarp_writer;
GRANT USAGE, SELECT ON SEQUENCE pgtimewarp.health_events_id_seq TO pgtimewarp_writer;

-- Reader role: CLI and extension read-only access
CREATE ROLE pgtimewarp_reader;

GRANT USAGE ON SCHEMA pgtimewarp TO pgtimewarp_reader;

GRANT SELECT ON pgtimewarp.nodes TO pgtimewarp_reader;
GRANT SELECT ON pgtimewarp.tracked_relations TO pgtimewarp_reader;
GRANT SELECT ON pgtimewarp.row_versions TO pgtimewarp_reader;
GRANT SELECT ON pgtimewarp.lsn_time_map TO pgtimewarp_reader;
GRANT SELECT ON pgtimewarp.wal_checkpoints TO pgtimewarp_reader;
GRANT SELECT ON pgtimewarp.health_events TO pgtimewarp_reader;

-- Admin role: CLI manages tracking (track/untrack operations)
CREATE ROLE pgtimewarp_admin;

GRANT USAGE ON SCHEMA pgtimewarp TO pgtimewarp_admin;

GRANT SELECT, INSERT, UPDATE, DELETE ON pgtimewarp.tracked_relations TO pgtimewarp_admin;
GRANT SELECT ON pgtimewarp.nodes TO pgtimewarp_admin;
GRANT SELECT ON pgtimewarp.row_versions TO pgtimewarp_admin;
GRANT SELECT ON pgtimewarp.lsn_time_map TO pgtimewarp_admin;
GRANT SELECT ON pgtimewarp.wal_checkpoints TO pgtimewarp_admin;
GRANT SELECT ON pgtimewarp.health_events TO pgtimewarp_admin;
