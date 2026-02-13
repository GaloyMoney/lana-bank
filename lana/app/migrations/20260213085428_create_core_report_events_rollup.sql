-- Auto-generated rollup table for ReportEvent
CREATE TABLE core_report_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  external_id VARCHAR,
  files JSONB,
  name VARCHAR,
  norm VARCHAR,
  run_id UUID
,
  PRIMARY KEY (id, version)
);


-- Auto-generated trigger function for ReportEvent
CREATE OR REPLACE FUNCTION core_report_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_report_events_rollup%ROWTYPE;
  new_row core_report_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_report_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;
  new_row.event_type := NEW.event_type;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.external_id := (NEW.event ->> 'external_id');
    new_row.files := (NEW.event -> 'files');
    new_row.name := (NEW.event ->> 'name');
    new_row.norm := (NEW.event ->> 'norm');
    new_row.run_id := (NEW.event ->> 'run_id')::UUID;
  ELSE
    -- Default all fields to current values
    new_row.external_id := current_row.external_id;
    new_row.files := current_row.files;
    new_row.name := current_row.name;
    new_row.norm := current_row.norm;
    new_row.run_id := current_row.run_id;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.external_id := (NEW.event ->> 'external_id');
      new_row.files := (NEW.event -> 'files');
      new_row.name := (NEW.event ->> 'name');
      new_row.norm := (NEW.event ->> 'norm');
      new_row.run_id := (NEW.event ->> 'run_id')::UUID;
  END CASE;

  INSERT INTO core_report_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    external_id,
    files,
    name,
    norm,
    run_id
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.external_id,
    new_row.files,
    new_row.name,
    new_row.norm,
    new_row.run_id
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for ReportEvent
CREATE TRIGGER core_report_events_rollup_trigger
  AFTER INSERT ON core_report_events
  FOR EACH ROW
  EXECUTE FUNCTION core_report_events_rollup_trigger();
