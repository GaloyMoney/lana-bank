-- Auto-generated rollup table for PolicyEvent
CREATE TABLE core_policy_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  process_type VARCHAR,
  rules JSONB,

  -- Collection rollups
  audit_entry_ids BIGINT[]

);

-- Auto-generated trigger function for PolicyEvent
CREATE OR REPLACE FUNCTION core_policy_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_policy_events_rollup%ROWTYPE;
  new_row core_policy_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_policy_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'approval_rules_updated') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
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
    new_row.process_type := (NEW.event ->> 'process_type');
    new_row.rules := (NEW.event -> 'rules');
  ELSE
    -- Default all fields to current values
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.process_type := current_row.process_type;
    new_row.rules := current_row.rules;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.process_type := (NEW.event ->> 'process_type');
      new_row.rules := (NEW.event -> 'rules');
    WHEN 'approval_rules_updated' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.rules := (NEW.event -> 'rules');
  END CASE;

  INSERT INTO core_policy_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    audit_entry_ids,
    process_type,
    rules
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.audit_entry_ids,
    new_row.process_type,
    new_row.rules
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    audit_entry_ids = EXCLUDED.audit_entry_ids,
    process_type = EXCLUDED.process_type,
    rules = EXCLUDED.rules;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for PolicyEvent
CREATE TRIGGER core_policy_events_rollup_trigger
  AFTER INSERT ON core_policy_events
  FOR EACH ROW
  EXECUTE FUNCTION core_policy_events_rollup_trigger();
