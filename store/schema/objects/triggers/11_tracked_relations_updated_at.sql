CREATE OR REPLACE FUNCTION pgtimewarp._touch_updated_at()
RETURNS trigger AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_trigger
    WHERE tgname = 'trg_tracked_relations_touch'
      AND tgrelid = 'pgtimewarp.tracked_relations'::regclass
  ) THEN
    CREATE TRIGGER trg_tracked_relations_touch
    BEFORE UPDATE ON pgtimewarp.tracked_relations
    FOR EACH ROW EXECUTE FUNCTION pgtimewarp._touch_updated_at();
  END IF;
END $$;
