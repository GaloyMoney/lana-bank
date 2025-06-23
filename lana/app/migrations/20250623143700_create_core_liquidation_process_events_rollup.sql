-- Auto-generated rollup table for LiquidationProcessEvent
CREATE TABLE core_liquidation_process_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  credit_facility_id UUID,
  effective VARCHAR,
  in_liquidation_account_id UUID,
  initial_amount JSONB,
  ledger_tx_id UUID,
  obligation_id UUID,

  -- Collection rollups
  audit_entry_ids BIGINT[],

  -- Toggle fields
  is_completed BOOLEAN DEFAULT false

);

-- Auto-generated trigger function for LiquidationProcessEvent
CREATE OR REPLACE FUNCTION core_liquidation_process_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_liquidation_process_events_rollup%ROWTYPE;
  new_row core_liquidation_process_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_liquidation_process_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'completed') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
    new_row.effective := (NEW.event ->> 'effective');
    new_row.in_liquidation_account_id := (NEW.event ->> 'in_liquidation_account_id')::UUID;
    new_row.initial_amount := (NEW.event -> 'initial_amount');
    new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
    new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.is_completed := false;
  ELSE
    -- Default all fields to current values
    new_row.credit_facility_id := current_row.credit_facility_id;
    new_row.effective := current_row.effective;
    new_row.in_liquidation_account_id := current_row.in_liquidation_account_id;
    new_row.initial_amount := current_row.initial_amount;
    new_row.ledger_tx_id := current_row.ledger_tx_id;
    new_row.obligation_id := current_row.obligation_id;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.is_completed := current_row.is_completed;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
      new_row.effective := (NEW.event ->> 'effective');
      new_row.in_liquidation_account_id := (NEW.event ->> 'in_liquidation_account_id')::UUID;
      new_row.initial_amount := (NEW.event -> 'initial_amount');
      new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
      new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'completed' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_completed := true;
  END CASE;

  INSERT INTO core_liquidation_process_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    credit_facility_id,
    effective,
    in_liquidation_account_id,
    initial_amount,
    ledger_tx_id,
    obligation_id,
    audit_entry_ids,
    is_completed
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.credit_facility_id,
    new_row.effective,
    new_row.in_liquidation_account_id,
    new_row.initial_amount,
    new_row.ledger_tx_id,
    new_row.obligation_id,
    new_row.audit_entry_ids,
    new_row.is_completed
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    credit_facility_id = EXCLUDED.credit_facility_id,
    effective = EXCLUDED.effective,
    in_liquidation_account_id = EXCLUDED.in_liquidation_account_id,
    initial_amount = EXCLUDED.initial_amount,
    ledger_tx_id = EXCLUDED.ledger_tx_id,
    obligation_id = EXCLUDED.obligation_id,
    audit_entry_ids = EXCLUDED.audit_entry_ids,
    is_completed = EXCLUDED.is_completed;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for LiquidationProcessEvent
CREATE TRIGGER core_liquidation_process_events_rollup_trigger
  AFTER INSERT ON core_liquidation_process_events
  FOR EACH ROW
  EXECUTE FUNCTION core_liquidation_process_events_rollup_trigger();
