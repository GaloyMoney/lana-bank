-- Auto-generated rollup table for ReportRunEvent
CREATE TABLE core_report_run_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  external_id VARCHAR,
  run_type VARCHAR,
  start_time TIMESTAMPTZ,
  state VARCHAR
,
  PRIMARY KEY (id, version)
);


-- Auto-generated trigger function for ReportRunEvent
CREATE OR REPLACE FUNCTION core_report_run_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_report_run_events_rollup%ROWTYPE;
  new_row core_report_run_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_report_run_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'state_updated') THEN
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
    new_row.run_type := (NEW.event ->> 'run_type');
    new_row.start_time := (NEW.event ->> 'start_time')::TIMESTAMPTZ;
    new_row.state := (NEW.event ->> 'state');
  ELSE
    -- Default all fields to current values
    new_row.external_id := current_row.external_id;
    new_row.run_type := current_row.run_type;
    new_row.start_time := current_row.start_time;
    new_row.state := current_row.state;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.external_id := (NEW.event ->> 'external_id');
      new_row.run_type := (NEW.event ->> 'run_type');
      new_row.start_time := (NEW.event ->> 'start_time')::TIMESTAMPTZ;
      new_row.state := (NEW.event ->> 'state');
    WHEN 'state_updated' THEN
      new_row.run_type := (NEW.event ->> 'run_type');
      new_row.start_time := (NEW.event ->> 'start_time')::TIMESTAMPTZ;
      new_row.state := (NEW.event ->> 'state');
  END CASE;

  INSERT INTO core_report_run_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    external_id,
    run_type,
    start_time,
    state
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.external_id,
    new_row.run_type,
    new_row.start_time,
    new_row.state
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for ReportRunEvent
CREATE TRIGGER core_report_run_events_rollup_trigger
  AFTER INSERT ON core_report_run_events
  FOR EACH ROW
  EXECUTE FUNCTION core_report_run_events_rollup_trigger();
