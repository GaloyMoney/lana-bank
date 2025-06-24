-- Auto-generated rollup table for UserEvent
CREATE TABLE core_user_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  email VARCHAR,
  authentication_id UUID,
  role_id UUID,

  -- Collection rollups
  audit_entry_ids BIGINT[]

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

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_user_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'authentication_id_updated', 'role_granted', 'role_revoked') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.email := (NEW.event ->> 'email');
    new_row.authentication_id := (NEW.event ->> 'authentication_id')::UUID;
    new_row.role_id := CASE
       WHEN event_type = ANY(ARRAY['role_revoked']) THEN NULL
       ELSE (NEW.event ->> 'role_id')::UUID
     END;
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
  ELSE
    -- Default all fields to current values
    new_row.email := current_row.email;
    new_row.authentication_id := current_row.authentication_id;
    new_row.role_id := current_row.role_id;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.email := (NEW.event ->> 'email');
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'authentication_id_updated' THEN
      new_row.authentication_id := (NEW.event ->> 'authentication_id')::UUID;
    WHEN 'role_granted' THEN
      new_row.role_id := (NEW.event ->> 'role_id')::UUID;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'role_revoked' THEN
      new_row.role_id := NULL;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
  END CASE;

  INSERT INTO core_user_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    email,
    authentication_id,
    role_id,
    audit_entry_ids
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.email,
    new_row.authentication_id,
    new_row.role_id,
    new_row.audit_entry_ids
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    email = EXCLUDED.email,
    authentication_id = EXCLUDED.authentication_id,
    role_id = EXCLUDED.role_id,
    audit_entry_ids = EXCLUDED.audit_entry_ids;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for UserEvent
CREATE TRIGGER core_user_events_rollup_trigger
  AFTER INSERT ON core_user_events
  FOR EACH ROW
  EXECUTE FUNCTION core_user_events_rollup_trigger();
