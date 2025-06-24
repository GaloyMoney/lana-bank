-- Auto-generated rollup table for WithdrawalEvent
CREATE TABLE core_withdrawal_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  amount JSONB,
  approval_process_id UUID,
  deposit_account_id UUID,
  reference VARCHAR,
  approved BOOLEAN,
  ledger_tx_id UUID,

  -- Collection rollups
  audit_entry_ids BIGINT[],

  -- Toggle fields
  is_approval_process_concluded BOOLEAN DEFAULT false,
  is_confirmed BOOLEAN DEFAULT false,
  is_cancelled BOOLEAN DEFAULT false

);

-- Auto-generated trigger function for WithdrawalEvent
CREATE OR REPLACE FUNCTION core_withdrawal_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_withdrawal_events_rollup%ROWTYPE;
  new_row core_withdrawal_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_withdrawal_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'approval_process_concluded', 'confirmed', 'cancelled') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.amount := (NEW.event -> 'amount');
    new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
    new_row.deposit_account_id := (NEW.event ->> 'deposit_account_id')::UUID;
    new_row.reference := (NEW.event ->> 'reference');
    new_row.approved := (NEW.event ->> 'approved')::BOOLEAN;
    new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.is_approval_process_concluded := false;
    new_row.is_confirmed := false;
    new_row.is_cancelled := false;
  ELSE
    -- Default all fields to current values
    new_row.amount := current_row.amount;
    new_row.approval_process_id := current_row.approval_process_id;
    new_row.deposit_account_id := current_row.deposit_account_id;
    new_row.reference := current_row.reference;
    new_row.approved := current_row.approved;
    new_row.ledger_tx_id := current_row.ledger_tx_id;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.is_approval_process_concluded := current_row.is_approval_process_concluded;
    new_row.is_confirmed := current_row.is_confirmed;
    new_row.is_cancelled := current_row.is_cancelled;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.amount := (NEW.event -> 'amount');
      new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
      new_row.deposit_account_id := (NEW.event ->> 'deposit_account_id')::UUID;
      new_row.reference := (NEW.event ->> 'reference');
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'approval_process_concluded' THEN
      new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
      new_row.approved := (NEW.event ->> 'approved')::BOOLEAN;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_approval_process_concluded := true;
    WHEN 'confirmed' THEN
      new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_confirmed := true;
    WHEN 'cancelled' THEN
      new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_cancelled := true;
  END CASE;

  INSERT INTO core_withdrawal_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    amount,
    approval_process_id,
    deposit_account_id,
    reference,
    approved,
    ledger_tx_id,
    audit_entry_ids,
    is_approval_process_concluded,
    is_confirmed,
    is_cancelled
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.amount,
    new_row.approval_process_id,
    new_row.deposit_account_id,
    new_row.reference,
    new_row.approved,
    new_row.ledger_tx_id,
    new_row.audit_entry_ids,
    new_row.is_approval_process_concluded,
    new_row.is_confirmed,
    new_row.is_cancelled
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    amount = EXCLUDED.amount,
    approval_process_id = EXCLUDED.approval_process_id,
    deposit_account_id = EXCLUDED.deposit_account_id,
    reference = EXCLUDED.reference,
    approved = EXCLUDED.approved,
    ledger_tx_id = EXCLUDED.ledger_tx_id,
    audit_entry_ids = EXCLUDED.audit_entry_ids,
    is_approval_process_concluded = EXCLUDED.is_approval_process_concluded,
    is_confirmed = EXCLUDED.is_confirmed,
    is_cancelled = EXCLUDED.is_cancelled;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for WithdrawalEvent
CREATE TRIGGER core_withdrawal_events_rollup_trigger
  AFTER INSERT ON core_withdrawal_events
  FOR EACH ROW
  EXECUTE FUNCTION core_withdrawal_events_rollup_trigger();
