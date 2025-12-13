-- Auto-generated rollup table for FiscalYearEvent
CREATE TABLE core_fiscal_year_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  chart_id UUID,
  closed_as_of VARCHAR,
  closed_at TIMESTAMPTZ,
  month_closed_as_of VARCHAR,
  month_closed_at TIMESTAMPTZ,
  opened_as_of VARCHAR,
  reference VARCHAR
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for FiscalYearEvent
CREATE OR REPLACE FUNCTION core_fiscal_year_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_fiscal_year_events_rollup%ROWTYPE;
  new_row core_fiscal_year_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_fiscal_year_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'month_closed', 'year_closed') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.chart_id := (NEW.event ->> 'chart_id')::UUID;
    new_row.closed_as_of := (NEW.event ->> 'closed_as_of');
    new_row.closed_at := (NEW.event ->> 'closed_at')::TIMESTAMPTZ;
    new_row.month_closed_as_of := (NEW.event ->> 'month_closed_as_of');
    new_row.month_closed_at := (NEW.event ->> 'month_closed_at')::TIMESTAMPTZ;
    new_row.opened_as_of := (NEW.event ->> 'opened_as_of');
    new_row.reference := (NEW.event ->> 'reference');
  ELSE
    -- Default all fields to current values
    new_row.chart_id := current_row.chart_id;
    new_row.closed_as_of := current_row.closed_as_of;
    new_row.closed_at := current_row.closed_at;
    new_row.month_closed_as_of := current_row.month_closed_as_of;
    new_row.month_closed_at := current_row.month_closed_at;
    new_row.opened_as_of := current_row.opened_as_of;
    new_row.reference := current_row.reference;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.chart_id := (NEW.event ->> 'chart_id')::UUID;
      new_row.opened_as_of := (NEW.event ->> 'opened_as_of');
      new_row.reference := (NEW.event ->> 'reference');
    WHEN 'month_closed' THEN
      new_row.month_closed_as_of := (NEW.event ->> 'month_closed_as_of');
      new_row.month_closed_at := (NEW.event ->> 'month_closed_at')::TIMESTAMPTZ;
    WHEN 'year_closed' THEN
      new_row.closed_as_of := (NEW.event ->> 'closed_as_of');
      new_row.closed_at := (NEW.event ->> 'closed_at')::TIMESTAMPTZ;
  END CASE;

  INSERT INTO core_fiscal_year_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    chart_id,
    closed_as_of,
    closed_at,
    month_closed_as_of,
    month_closed_at,
    opened_as_of,
    reference
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.chart_id,
    new_row.closed_as_of,
    new_row.closed_at,
    new_row.month_closed_as_of,
    new_row.month_closed_at,
    new_row.opened_as_of,
    new_row.reference
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for FiscalYearEvent
CREATE TRIGGER core_fiscal_year_events_rollup_trigger
  AFTER INSERT ON core_fiscal_year_events
  FOR EACH ROW
  EXECUTE FUNCTION core_fiscal_year_events_rollup_trigger();
