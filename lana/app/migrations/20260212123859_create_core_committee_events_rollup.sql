-- Auto-generated rollup table for CommitteeEvent
CREATE TABLE core_committee_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  name VARCHAR,

  -- Collection rollups
  member_ids UUID[]
,
  PRIMARY KEY (id, version)
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

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_committee_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'member_added', 'member_removed') THEN
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
    new_row.member_ids := CASE
       WHEN NEW.event ? 'member_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'member_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.name := (NEW.event ->> 'name');
  ELSE
    -- Default all fields to current values
    new_row.member_ids := current_row.member_ids;
    new_row.name := current_row.name;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.name := (NEW.event ->> 'name');
    WHEN 'member_added' THEN
      new_row.member_ids := array_append(COALESCE(current_row.member_ids, ARRAY[]::UUID[]), (NEW.event ->> 'member_id')::UUID);
    WHEN 'member_removed' THEN
      new_row.member_ids := array_remove(COALESCE(current_row.member_ids, ARRAY[]::UUID[]), (NEW.event ->> 'member_id')::UUID);
  END CASE;

  INSERT INTO core_committee_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    member_ids,
    name
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.member_ids,
    new_row.name
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for CommitteeEvent
CREATE TRIGGER core_committee_events_rollup_trigger
  AFTER INSERT ON core_committee_events
  FOR EACH ROW
  EXECUTE FUNCTION core_committee_events_rollup_trigger();
