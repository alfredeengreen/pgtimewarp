CREATE OR REPLACE FUNCTION pgtimewarp._row_versions_consistency()
RETURNS trigger AS $$
DECLARE
  v_check_enabled BOOLEAN;
  v_node_id TEXT;
  v_schema_name TEXT;
  v_table_name TEXT;
  v_relid OID;
BEGIN
  v_check_enabled := COALESCE(
    current_setting('pgtimewarp.row_versions_consistency', true)::BOOLEAN,
    true
  );

  IF NOT v_check_enabled THEN
    RETURN NEW;
  END IF;
  SELECT node_id, schema_name, table_name, relid
    INTO v_node_id, v_schema_name, v_table_name, v_relid
  FROM pgtimewarp.tracked_relations
  WHERE id = NEW.tracked_id;

  IF v_node_id IS NULL THEN
    RAISE EXCEPTION 'tracked_relations % not found', NEW.tracked_id;
  END IF;

  IF NEW.node_id <> v_node_id THEN
    RAISE EXCEPTION 'row_versions.node_id % does not match tracked_relations.node_id %',
      NEW.node_id, v_node_id;
  END IF;

  IF NEW.schema_name <> v_schema_name THEN
    RAISE EXCEPTION 'row_versions.schema_name % does not match tracked_relations.schema_name %',
      NEW.schema_name, v_schema_name;
  END IF;

  IF NEW.table_name <> v_table_name THEN
    RAISE EXCEPTION 'row_versions.table_name % does not match tracked_relations.table_name %',
      NEW.table_name, v_table_name;
  END IF;

  IF NEW.relid IS NOT NULL AND v_relid IS NOT NULL AND NEW.relid <> v_relid THEN
    RAISE EXCEPTION 'row_versions.relid % does not match tracked_relations.relid %',
      NEW.relid, v_relid;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_trigger
    WHERE tgname = 'trg_row_versions_consistency'
      AND tgrelid = 'pgtimewarp.row_versions'::regclass
  ) THEN
    CREATE TRIGGER trg_row_versions_consistency
    BEFORE INSERT OR UPDATE ON pgtimewarp.row_versions
    FOR EACH ROW EXECUTE FUNCTION pgtimewarp._row_versions_consistency();
  END IF;
END $$;
