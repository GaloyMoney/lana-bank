-- Auto-generated rollup table for PaymentAllocationEvent
CREATE TABLE core_payment_allocation_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  account_to_be_debited_id UUID,
  amount JSONB,
  credit_facility_id UUID,
  effective VARCHAR,
  ledger_tx_id UUID,
  obligation_allocation_idx INTEGER,
  obligation_id UUID,
  obligation_type JSONB,
  payment_id UUID,
  receivable_account_id UUID,

  -- Collection rollups
  audit_entry_ids BIGINT[]

);

-- Auto-generated trigger function for PaymentAllocationEvent
CREATE OR REPLACE FUNCTION core_payment_allocation_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_payment_allocation_events_rollup%ROWTYPE;
  new_row core_payment_allocation_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_payment_allocation_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.account_to_be_debited_id := (NEW.event ->> 'account_to_be_debited_id')::UUID;
    new_row.amount := (NEW.event -> 'amount');
    new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
    new_row.effective := (NEW.event ->> 'effective');
    new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
    new_row.obligation_allocation_idx := (NEW.event ->> 'obligation_allocation_idx')::INTEGER;
    new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
    new_row.obligation_type := (NEW.event -> 'obligation_type');
    new_row.payment_id := (NEW.event ->> 'payment_id')::UUID;
    new_row.receivable_account_id := (NEW.event ->> 'receivable_account_id')::UUID;
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
  ELSE
    -- Default all fields to current values
    new_row.account_to_be_debited_id := current_row.account_to_be_debited_id;
    new_row.amount := current_row.amount;
    new_row.credit_facility_id := current_row.credit_facility_id;
    new_row.effective := current_row.effective;
    new_row.ledger_tx_id := current_row.ledger_tx_id;
    new_row.obligation_allocation_idx := current_row.obligation_allocation_idx;
    new_row.obligation_id := current_row.obligation_id;
    new_row.obligation_type := current_row.obligation_type;
    new_row.payment_id := current_row.payment_id;
    new_row.receivable_account_id := current_row.receivable_account_id;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_to_be_debited_id := (NEW.event ->> 'account_to_be_debited_id')::UUID;
      new_row.amount := (NEW.event -> 'amount');
      new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
      new_row.effective := (NEW.event ->> 'effective');
      new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
      new_row.obligation_allocation_idx := (NEW.event ->> 'obligation_allocation_idx')::INTEGER;
      new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
      new_row.obligation_type := (NEW.event -> 'obligation_type');
      new_row.payment_id := (NEW.event ->> 'payment_id')::UUID;
      new_row.receivable_account_id := (NEW.event ->> 'receivable_account_id')::UUID;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
  END CASE;

  INSERT INTO core_payment_allocation_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    account_to_be_debited_id,
    amount,
    credit_facility_id,
    effective,
    ledger_tx_id,
    obligation_allocation_idx,
    obligation_id,
    obligation_type,
    payment_id,
    receivable_account_id,
    audit_entry_ids
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.account_to_be_debited_id,
    new_row.amount,
    new_row.credit_facility_id,
    new_row.effective,
    new_row.ledger_tx_id,
    new_row.obligation_allocation_idx,
    new_row.obligation_id,
    new_row.obligation_type,
    new_row.payment_id,
    new_row.receivable_account_id,
    new_row.audit_entry_ids
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    account_to_be_debited_id = EXCLUDED.account_to_be_debited_id,
    amount = EXCLUDED.amount,
    credit_facility_id = EXCLUDED.credit_facility_id,
    effective = EXCLUDED.effective,
    ledger_tx_id = EXCLUDED.ledger_tx_id,
    obligation_allocation_idx = EXCLUDED.obligation_allocation_idx,
    obligation_id = EXCLUDED.obligation_id,
    obligation_type = EXCLUDED.obligation_type,
    payment_id = EXCLUDED.payment_id,
    receivable_account_id = EXCLUDED.receivable_account_id,
    audit_entry_ids = EXCLUDED.audit_entry_ids;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for PaymentAllocationEvent
CREATE TRIGGER core_payment_allocation_events_rollup_trigger
  AFTER INSERT ON core_payment_allocation_events
  FOR EACH ROW
  EXECUTE FUNCTION core_payment_allocation_events_rollup_trigger();
