-- Auto-generated rollup table for ApprovalProcessEvent
CREATE TABLE core_approval_process_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  policy_id UUID,
  process_type VARCHAR,
  rules JSONB,
  target_ref VARCHAR,
  approved BOOLEAN,

  -- Collection rollups
  approver_ids UUID[],
  deny_reasons VARCHAR[],
  audit_entry_ids BIGINT[],
  denier_ids UUID[],

  -- Toggle fields
  is_concluded BOOLEAN DEFAULT false

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

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_approval_process_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'approved', 'denied', 'concluded') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.policy_id := (NEW.event ->> 'policy_id')::UUID;
    new_row.process_type := (NEW.event ->> 'process_type');
    new_row.rules := (NEW.event -> 'rules');
    new_row.target_ref := (NEW.event ->> 'target_ref');
    new_row.approved := (NEW.event ->> 'approved')::BOOLEAN;
    new_row.approver_ids := CASE
       WHEN NEW.event ? 'approver_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'approver_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.deny_reasons := CASE
       WHEN NEW.event ? 'deny_reasons' THEN
         ARRAY(SELECT value::text FROM jsonb_array_elements_text(NEW.event -> 'deny_reasons'))
       ELSE ARRAY[]::VARCHAR[]
     END
;
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.denier_ids := CASE
       WHEN NEW.event ? 'denier_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'denier_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.is_concluded := false;
  ELSE
    -- Default all fields to current values
    new_row.policy_id := current_row.policy_id;
    new_row.process_type := current_row.process_type;
    new_row.rules := current_row.rules;
    new_row.target_ref := current_row.target_ref;
    new_row.approved := current_row.approved;
    new_row.approver_ids := current_row.approver_ids;
    new_row.deny_reasons := current_row.deny_reasons;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.denier_ids := current_row.denier_ids;
    new_row.is_concluded := current_row.is_concluded;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.policy_id := (NEW.event ->> 'policy_id')::UUID;
      new_row.process_type := (NEW.event ->> 'process_type');
      new_row.rules := (NEW.event -> 'rules');
      new_row.target_ref := (NEW.event ->> 'target_ref');
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'approved' THEN
      new_row.approver_ids := array_append(COALESCE(current_row.approver_ids, ARRAY[]::UUID[]), (NEW.event ->> 'approver_id')::UUID);
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'denied' THEN
      new_row.deny_reasons := array_append(COALESCE(current_row.deny_reasons, ARRAY[]::VARCHAR[]), (NEW.event ->> 'reason'));
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.denier_ids := array_append(COALESCE(current_row.denier_ids, ARRAY[]::UUID[]), (NEW.event ->> 'denier_id')::UUID);
    WHEN 'concluded' THEN
      new_row.approved := (NEW.event ->> 'approved')::BOOLEAN;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_concluded := true;
  END CASE;

  INSERT INTO core_approval_process_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    policy_id,
    process_type,
    rules,
    target_ref,
    approved,
    approver_ids,
    deny_reasons,
    audit_entry_ids,
    denier_ids,
    is_concluded
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.policy_id,
    new_row.process_type,
    new_row.rules,
    new_row.target_ref,
    new_row.approved,
    new_row.approver_ids,
    new_row.deny_reasons,
    new_row.audit_entry_ids,
    new_row.denier_ids,
    new_row.is_concluded
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    policy_id = EXCLUDED.policy_id,
    process_type = EXCLUDED.process_type,
    rules = EXCLUDED.rules,
    target_ref = EXCLUDED.target_ref,
    approved = EXCLUDED.approved,
    approver_ids = EXCLUDED.approver_ids,
    deny_reasons = EXCLUDED.deny_reasons,
    audit_entry_ids = EXCLUDED.audit_entry_ids,
    denier_ids = EXCLUDED.denier_ids,
    is_concluded = EXCLUDED.is_concluded;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for ApprovalProcessEvent
CREATE TRIGGER core_approval_process_events_rollup_trigger
  AFTER INSERT ON core_approval_process_events
  FOR EACH ROW
  EXECUTE FUNCTION core_approval_process_events_rollup_trigger();
