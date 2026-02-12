-- Auto-generated rollup table for PermissionSetEvent
CREATE TABLE core_permission_set_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  name VARCHAR,
  permissions JSONB
,
  PRIMARY KEY (id, version)
);


-- Auto-generated trigger function for PermissionSetEvent
CREATE OR REPLACE FUNCTION core_permission_set_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_permission_set_events_rollup%ROWTYPE;
  new_row core_permission_set_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_permission_set_events_rollup
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
  new_row.event_type := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.name := (NEW.event ->> 'name');
    new_row.permissions := (NEW.event -> 'permissions');
  ELSE
    -- Default all fields to current values
    new_row.name := current_row.name;
    new_row.permissions := current_row.permissions;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.name := (NEW.event ->> 'name');
      new_row.permissions := (NEW.event -> 'permissions');
  END CASE;

  INSERT INTO core_permission_set_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    name,
    permissions
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.name,
    new_row.permissions
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for PermissionSetEvent
CREATE TRIGGER core_permission_set_events_rollup_trigger
  AFTER INSERT ON core_permission_set_events
  FOR EACH ROW
  EXECUTE FUNCTION core_permission_set_events_rollup_trigger();
