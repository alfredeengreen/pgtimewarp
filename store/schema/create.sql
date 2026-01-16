\set ON_ERROR_STOP on

\ir objects/00_schema.sql
\ir objects/05_extensions.sql

\ir objects/10_nodes.sql
\ir objects/11_tracked_relations.sql
\ir objects/12_row_versions.sql
\ir objects/13_lsn_time_map.sql
\ir objects/14_wal_checkpoints.sql
\ir objects/15_health_events.sql

\ir objects/indexes/10_nodes.sql
\ir objects/indexes/11_tracked_relations.sql
\ir objects/indexes/12_row_versions.sql
\ir objects/indexes/13_lsn_time_map.sql
\ir objects/indexes/15_health_events.sql

\ir objects/triggers/11_tracked_relations_updated_at.sql
\ir objects/triggers/12_row_versions_consistency.sql

\ir objects/roles/pgtimewarp_writer.sql
\ir objects/roles/pgtimewarp_reader.sql
\ir objects/roles/pgtimewarp_admin.sql

\ir objects/functions/10_retention.sql
