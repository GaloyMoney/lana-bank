-- Auto-generated rollup table for ApprovalProcessEvent
CREATE TABLE core_approval_process_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  approved BOOLEAN,
  policy_id UUID,
  process_type VARCHAR,
  rules JSONB,
  target_ref VARCHAR,

  -- Collection rollups
  approver_ids UUID[],
  denier_ids UUID[],
  deny_reasons VARCHAR[],

  -- Toggle fields
  is_concluded BOOLEAN DEFAULT false
,
  PRIMARY KEY (id, version)
);


-- Auto-generated trigger function for ApprovalProcessEvent
CREATE OR REPLACE FUNCTION core_approval_process_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_approval_process_events_rollup%ROWTYPE;
  new_row core_approval_process_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_approval_process_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'approved', 'denied', 'concluded') THEN
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
    new_row.approved := (NEW.event ->> 'approved')::BOOLEAN;
    new_row.approver_ids := CASE
       WHEN NEW.event ? 'approver_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'approver_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.denier_ids := CASE
       WHEN NEW.event ? 'denier_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'denier_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.deny_reasons := CASE
       WHEN NEW.event ? 'deny_reasons' THEN
         ARRAY(SELECT value::text FROM jsonb_array_elements_text(NEW.event -> 'deny_reasons'))
       ELSE ARRAY[]::VARCHAR[]
     END
;
    new_row.is_concluded := false;
    new_row.policy_id := (NEW.event ->> 'policy_id')::UUID;
    new_row.process_type := (NEW.event ->> 'process_type');
    new_row.rules := (NEW.event -> 'rules');
    new_row.target_ref := (NEW.event ->> 'target_ref');
  ELSE
    -- Default all fields to current values
    new_row.approved := current_row.approved;
    new_row.approver_ids := current_row.approver_ids;
    new_row.denier_ids := current_row.denier_ids;
    new_row.deny_reasons := current_row.deny_reasons;
    new_row.is_concluded := current_row.is_concluded;
    new_row.policy_id := current_row.policy_id;
    new_row.process_type := current_row.process_type;
    new_row.rules := current_row.rules;
    new_row.target_ref := current_row.target_ref;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.policy_id := (NEW.event ->> 'policy_id')::UUID;
      new_row.process_type := (NEW.event ->> 'process_type');
      new_row.rules := (NEW.event -> 'rules');
      new_row.target_ref := (NEW.event ->> 'target_ref');
    WHEN 'approved' THEN
      new_row.approver_ids := array_append(COALESCE(current_row.approver_ids, ARRAY[]::UUID[]), (NEW.event ->> 'approver_id')::UUID);
    WHEN 'denied' THEN
      new_row.denier_ids := array_append(COALESCE(current_row.denier_ids, ARRAY[]::UUID[]), (NEW.event ->> 'denier_id')::UUID);
      new_row.deny_reasons := array_append(COALESCE(current_row.deny_reasons, ARRAY[]::VARCHAR[]), (NEW.event ->> 'reason'));
    WHEN 'concluded' THEN
      new_row.approved := (NEW.event ->> 'approved')::BOOLEAN;
      new_row.is_concluded := true;
  END CASE;

  INSERT INTO core_approval_process_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    approved,
    approver_ids,
    denier_ids,
    deny_reasons,
    is_concluded,
    policy_id,
    process_type,
    rules,
    target_ref
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.approved,
    new_row.approver_ids,
    new_row.denier_ids,
    new_row.deny_reasons,
    new_row.is_concluded,
    new_row.policy_id,
    new_row.process_type,
    new_row.rules,
    new_row.target_ref
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for ApprovalProcessEvent
CREATE TRIGGER core_approval_process_events_rollup_trigger
  AFTER INSERT ON core_approval_process_events
  FOR EACH ROW
  EXECUTE FUNCTION core_approval_process_events_rollup_trigger();
