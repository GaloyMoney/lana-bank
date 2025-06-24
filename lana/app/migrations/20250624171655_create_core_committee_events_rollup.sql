-- Auto-generated rollup table for CommitteeEvent
CREATE TABLE core_committee_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  name VARCHAR,

  -- Collection rollups
  audit_entry_ids BIGINT[],
  member_ids UUID[]

);

-- Auto-generated trigger function for CommitteeEvent
CREATE OR REPLACE FUNCTION core_committee_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_committee_events_rollup%ROWTYPE;
  new_row core_committee_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_committee_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'member_added', 'member_removed') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.name := (NEW.event ->> 'name');
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.member_ids := CASE
       WHEN NEW.event ? 'member_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'member_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
  ELSE
    -- Default all fields to current values
    new_row.name := current_row.name;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.member_ids := current_row.member_ids;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.name := (NEW.event ->> 'name');
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'member_added' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.member_ids := array_append(COALESCE(current_row.member_ids, ARRAY[]::UUID[]), (NEW.event ->> 'member_id')::UUID);
    WHEN 'member_removed' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.member_ids := array_remove(COALESCE(current_row.member_ids, ARRAY[]::UUID[]), (NEW.event ->> 'member_id')::UUID);
  END CASE;

  INSERT INTO core_committee_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    name,
    audit_entry_ids,
    member_ids
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.name,
    new_row.audit_entry_ids,
    new_row.member_ids
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    name = EXCLUDED.name,
    audit_entry_ids = EXCLUDED.audit_entry_ids,
    member_ids = EXCLUDED.member_ids;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for CommitteeEvent
CREATE TRIGGER core_committee_events_rollup_trigger
  AFTER INSERT ON core_committee_events
  FOR EACH ROW
  EXECUTE FUNCTION core_committee_events_rollup_trigger();
