-- Auto-generated rollup table for UserEvent
CREATE TABLE core_user_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  authentication_id UUID,
  email VARCHAR,
  role_id UUID,

  -- Collection rollups
  audit_entry_ids BIGINT[]
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for UserEvent
CREATE OR REPLACE FUNCTION core_user_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_user_events_rollup%ROWTYPE;
  new_row core_user_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_user_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'authentication_id_updated', 'role_granted', 'role_revoked') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.authentication_id := (NEW.event ->> 'authentication_id')::UUID;
    new_row.email := (NEW.event ->> 'email');
    new_row.role_id := CASE
       WHEN event_type = ANY(ARRAY['role_revoked']) THEN NULL
       ELSE (NEW.event ->> 'role_id')::UUID
     END;
  ELSE
    -- Default all fields to current values
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.authentication_id := current_row.authentication_id;
    new_row.email := current_row.email;
    new_row.role_id := current_row.role_id;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.email := (NEW.event ->> 'email');
    WHEN 'authentication_id_updated' THEN
      new_row.authentication_id := (NEW.event ->> 'authentication_id')::UUID;
    WHEN 'role_granted' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.role_id := (NEW.event ->> 'role_id')::UUID;
    WHEN 'role_revoked' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.role_id := NULL;
  END CASE;

  INSERT INTO core_user_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    audit_entry_ids,
    authentication_id,
    email,
    role_id
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.audit_entry_ids,
    new_row.authentication_id,
    new_row.email,
    new_row.role_id
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for UserEvent
CREATE TRIGGER core_user_events_rollup_trigger
  AFTER INSERT ON core_user_events
  FOR EACH ROW
  EXECUTE FUNCTION core_user_events_rollup_trigger();
