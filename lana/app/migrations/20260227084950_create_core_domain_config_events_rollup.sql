-- Auto-generated rollup table for DomainConfigEvent
CREATE TABLE core_domain_config_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  config_type VARCHAR,
  encrypted BOOLEAN,
  key VARCHAR,
  value JSONB,
  visibility VARCHAR
,
  PRIMARY KEY (id, version)
);


-- Auto-generated trigger function for DomainConfigEvent
CREATE OR REPLACE FUNCTION core_domain_config_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_domain_config_events_rollup%ROWTYPE;
  new_row core_domain_config_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_domain_config_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'updated', 'key_rotated') THEN
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
    new_row.config_type := (NEW.event ->> 'config_type');
    new_row.encrypted := (NEW.event ->> 'encrypted')::BOOLEAN;
    new_row.key := (NEW.event ->> 'key');
    new_row.value := (NEW.event -> 'value');
    new_row.visibility := (NEW.event ->> 'visibility');
  ELSE
    -- Default all fields to current values
    new_row.config_type := current_row.config_type;
    new_row.encrypted := current_row.encrypted;
    new_row.key := current_row.key;
    new_row.value := current_row.value;
    new_row.visibility := current_row.visibility;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.config_type := (NEW.event ->> 'config_type');
      new_row.encrypted := (NEW.event ->> 'encrypted')::BOOLEAN;
      new_row.key := (NEW.event ->> 'key');
      new_row.visibility := (NEW.event ->> 'visibility');
    WHEN 'updated' THEN
      new_row.value := (NEW.event -> 'value');
    WHEN 'key_rotated' THEN
      new_row.value := (NEW.event -> 'value');
  END CASE;

  INSERT INTO core_domain_config_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    config_type,
    encrypted,
    key,
    value,
    visibility
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.config_type,
    new_row.encrypted,
    new_row.key,
    new_row.value,
    new_row.visibility
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for DomainConfigEvent
CREATE TRIGGER core_domain_config_events_rollup_trigger
  AFTER INSERT ON core_domain_config_events
  FOR EACH ROW
  EXECUTE FUNCTION core_domain_config_events_rollup_trigger();
