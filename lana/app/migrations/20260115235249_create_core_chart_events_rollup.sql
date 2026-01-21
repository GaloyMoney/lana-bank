-- Auto-generated rollup table for ChartEvent
CREATE TABLE core_chart_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  account_set_id UUID,
  base_config JSONB,
  closed_as_of VARCHAR,
  name VARCHAR,
  posted_as_of VARCHAR,
  reference VARCHAR
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for ChartEvent
CREATE OR REPLACE FUNCTION core_chart_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_chart_events_rollup%ROWTYPE;
  new_row core_chart_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_chart_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'base_config_set', 'closed_as_of', 'closing_transaction_posted') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.account_set_id := (NEW.event ->> 'account_set_id')::UUID;
    new_row.base_config := (NEW.event -> 'base_config');
    new_row.closed_as_of := (NEW.event ->> 'closed_as_of');
    new_row.name := (NEW.event ->> 'name');
    new_row.posted_as_of := (NEW.event ->> 'posted_as_of');
    new_row.reference := (NEW.event ->> 'reference');
  ELSE
    -- Default all fields to current values
    new_row.account_set_id := current_row.account_set_id;
    new_row.base_config := current_row.base_config;
    new_row.closed_as_of := current_row.closed_as_of;
    new_row.name := current_row.name;
    new_row.posted_as_of := current_row.posted_as_of;
    new_row.reference := current_row.reference;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_set_id := (NEW.event ->> 'account_set_id')::UUID;
      new_row.name := (NEW.event ->> 'name');
      new_row.reference := (NEW.event ->> 'reference');
    WHEN 'base_config_set' THEN
      new_row.base_config := (NEW.event -> 'base_config');
    WHEN 'closed_as_of' THEN
      new_row.closed_as_of := (NEW.event ->> 'closed_as_of');
    WHEN 'closing_transaction_posted' THEN
      new_row.posted_as_of := (NEW.event ->> 'posted_as_of');
  END CASE;

  INSERT INTO core_chart_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    account_set_id,
    base_config,
    closed_as_of,
    name,
    posted_as_of,
    reference
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.account_set_id,
    new_row.base_config,
    new_row.closed_as_of,
    new_row.name,
    new_row.posted_as_of,
    new_row.reference
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for ChartEvent
CREATE TRIGGER core_chart_events_rollup_trigger
  AFTER INSERT ON core_chart_events
  FOR EACH ROW
  EXECUTE FUNCTION core_chart_events_rollup_trigger();
