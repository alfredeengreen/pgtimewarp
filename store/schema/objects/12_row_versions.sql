CREATE TABLE IF NOT EXISTS pgtimewarp.row_versions (
  id              BIGSERIAL PRIMARY KEY,

  -- Stable relation identity in store DB
  tracked_id      BIGINT NOT NULL REFERENCES pgtimewarp.tracked_relations(id) ON DELETE CASCADE,

  -- Redundant for easy filtering (kept consistent via FK join in writes)
  node_id         TEXT NOT NULL,
  schema_name     TEXT NOT NULL,
  table_name      TEXT NOT NULL,

  -- Optional cache; not authoritative
  relid           OID NULL,

  -- Identity of row (hash of canonical PK bytes)
  pk_hash         BIGINT NOT NULL,

  -- Best-effort wall clock (for UX); authoritative order is LSN
  valid_from_ts   TIMESTAMPTZ NOT NULL,
  valid_to_ts     TIMESTAMPTZ NULL,

  valid_from_lsn  PG_LSN NOT NULL,
  valid_to_lsn    PG_LSN NULL,

  -- 0=insert,1=update,2=delete
  op              SMALLINT NOT NULL CHECK (op IN (0,1,2)),

  -- Full row image at that version (NULL allowed for tombstones if you want)
  row_data        JSONB NULL,

  txid            BIGINT NULL,
  confidence      SMALLINT NOT NULL DEFAULT 2 CHECK (confidence IN (0,1,2)),
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

  -- Sanity: if to_* set then it must be after from_*
  CHECK (valid_to_lsn IS NULL OR valid_to_lsn > valid_from_lsn),
  CHECK (valid_to_ts  IS NULL OR valid_to_ts  > valid_from_ts)
);

-- Correctness invariant: at most one open-ended version per (tracked_id, pk_hash)
CREATE UNIQUE INDEX IF NOT EXISTS uq_row_versions_open
  ON pgtimewarp.row_versions (tracked_id, pk_hash)
  WHERE valid_to_lsn IS NULL;

-- Optional: prevent duplicate versions at same LSN (usually indicates decoder bug)
CREATE UNIQUE INDEX IF NOT EXISTS uq_row_versions_version
  ON pgtimewarp.row_versions (tracked_id, pk_hash, valid_from_lsn);
