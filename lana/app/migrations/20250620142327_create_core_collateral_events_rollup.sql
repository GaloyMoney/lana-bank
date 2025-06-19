-- Auto-generated rollup table for CollateralEvent
CREATE TABLE core_collateral_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  account_id UUID,
  credit_facility_id UUID,
  abs_diff JSONB,
  action JSONB,
  collateral_amount JSONB,

  -- Collection rollups
  ledger_tx_ids UUID[],
  audit_entry_ids BIGINT[]

);

-- Auto-generated trigger function for CollateralEvent
CREATE OR REPLACE FUNCTION core_collateral_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_collateral_events_rollup%ROWTYPE;
  new_row core_collateral_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_collateral_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'updated') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.account_id := (NEW.event ->> 'account_id')::UUID;
    new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
    new_row.abs_diff := (NEW.event -> 'abs_diff');
    new_row.action := (NEW.event -> 'action');
    new_row.collateral_amount := (NEW.event -> 'collateral_amount');
    new_row.ledger_tx_ids := CASE
       WHEN NEW.event ? 'ledger_tx_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'ledger_tx_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
  ELSE
    -- Default all fields to current values
    new_row.account_id := current_row.account_id;
    new_row.credit_facility_id := current_row.credit_facility_id;
    new_row.abs_diff := current_row.abs_diff;
    new_row.action := current_row.action;
    new_row.collateral_amount := current_row.collateral_amount;
    new_row.ledger_tx_ids := current_row.ledger_tx_ids;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_id := (NEW.event ->> 'account_id')::UUID;
      new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
    WHEN 'updated' THEN
      new_row.abs_diff := (NEW.event -> 'abs_diff');
      new_row.action := (NEW.event -> 'action');
      new_row.collateral_amount := (NEW.event -> 'collateral_amount');
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
  END CASE;

  INSERT INTO core_collateral_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    account_id,
    credit_facility_id,
    abs_diff,
    action,
    collateral_amount,
    ledger_tx_ids,
    audit_entry_ids
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.account_id,
    new_row.credit_facility_id,
    new_row.abs_diff,
    new_row.action,
    new_row.collateral_amount,
    new_row.ledger_tx_ids,
    new_row.audit_entry_ids
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    account_id = EXCLUDED.account_id,
    credit_facility_id = EXCLUDED.credit_facility_id,
    abs_diff = EXCLUDED.abs_diff,
    action = EXCLUDED.action,
    collateral_amount = EXCLUDED.collateral_amount,
    ledger_tx_ids = EXCLUDED.ledger_tx_ids,
    audit_entry_ids = EXCLUDED.audit_entry_ids;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for CollateralEvent
CREATE TRIGGER core_collateral_events_rollup_trigger
  AFTER INSERT ON core_collateral_events
  FOR EACH ROW
  EXECUTE FUNCTION core_collateral_events_rollup_trigger();
